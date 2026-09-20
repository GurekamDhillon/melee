//===- gwtool.cpp - Melee PC port "GameCube-shaped memory" IR transformer -===//
//
// Input : LLVM IR or bitcode produced by
//           clang --target=ppc32-none-eabi -O2 -Xclang -disable-llvm-passes -emit-llvm -c
//         (big-endian data layout, 32-bit pointers, PowerPC SysV struct/bitfield layout).
// Output: i686-pc-windows-msvc code (COFF object by default) in which all memory touched by
//         game code keeps the exact GameCube byte image:
//
//  * Every load/store of a multi-byte scalar (integer, float, double, pointer, or a vector of
//    those) goes through llvm.bswap: values are big-endian in memory and native in registers.
//    Disc data, textures, vertex arrays and display lists need no conversion, and HSD archive
//    relocation runs unmodified.
//  * Scalar constants in global initializers are emitted pre-swapped. Scalars holding a
//    link-time address (pointers, ptrtoint expressions) cannot be swapped statically; their
//    locations go into a fixup table in section ".gwfix$m", which the runtime walks once at
//    startup (bracketed by ".gwfix$a"/".gwfix$z" markers) to swap each slot in place. Such
//    globals are marked externally_initialized so the optimizer never folds their initializer.
//  * Every external symbol gets a prefix (default "gw_") so game code never collides with
//    native code. Game<->game references resolve among game objects; every SDK/libc function
//    the game uses must be provided by a native shim named gw_<name>. Names starting with
//    "__gwrt_" are runtime hooks and keep their name.
//  * Integer division/remainder never traps (PowerPC divw/divwu semantics), variable shifts by
//    >= the bit width follow slw/srw/sraw, and float->int conversions saturate like fctiwz.
//  * The module is retargeted to i686-pc-windows-msvc after verifying that every aggregate
//    layout is identical under the PowerPC and x86 data layouts, then optimized and emitted.
//===----------------------------------------------------------------------===//

#include "llvm/ADT/DenseMap.h"
#include "llvm/ADT/DenseSet.h"
#include "llvm/ADT/SmallVector.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/Bitcode/BitcodeWriter.h"
#include "llvm/IR/Constants.h"
#include "llvm/IR/DataLayout.h"
#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/GlobalVariable.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/InstIterator.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/IntrinsicInst.h"
#include "llvm/IR/Intrinsics.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/Verifier.h"
#include "llvm/IRReader/IRReader.h"
#include "llvm/MC/TargetRegistry.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Transforms/IPO/GlobalDCE.h"
#include "llvm/ADT/StringMap.h"
#include "llvm/Support/CommandLine.h"
#include "llvm/Support/FileSystem.h"
#include "llvm/Support/InitLLVM.h"
#include "llvm/Support/SourceMgr.h"
#include "llvm/Support/TargetSelect.h"
#include "llvm/Support/ToolOutputFile.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/Target/TargetMachine.h"
#include "llvm/Target/TargetOptions.h"
#include "llvm/TargetParser/Triple.h"
#include "llvm/Transforms/Utils/ModuleUtils.h"

#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

using namespace llvm;

namespace {

cl::opt<std::string> InputFilename(cl::Positional, cl::desc("<input .bc/.ll>"), cl::Required);
cl::opt<std::string> OutputFilename("o", cl::desc("Output file"), cl::value_desc("file"),
                                    cl::Required);
cl::opt<std::string> EmitKind("emit", cl::desc("Output kind: obj|asm|bc|ll"), cl::init("obj"));
cl::opt<unsigned> OptLevelOpt("opt", cl::desc("Optimization level 0-3"), cl::init(2));
cl::opt<std::string> SymPrefix("prefix", cl::desc("External symbol prefix"), cl::init("gw_"));
cl::opt<std::string> ImportsFile("imports",
                                 cl::desc("Write undefined external symbols to this file"),
                                 cl::init(""));
cl::opt<std::string> CPUName("mcpu", cl::desc("x86 CPU"), cl::init("pentium4"));
cl::opt<std::string> FeatureStr("mattr", cl::desc("x86 features"),
                                cl::init("+sse,+sse2,+cmov,+cx8,+mmx,+fxsr"));
cl::opt<bool> NoSafeArith("no-safe-arith",
                          cl::desc("Keep trapping division, UB shifts and poison fptosi"),
                          cl::init(false));
cl::opt<bool> NoSwap("no-swap", cl::desc("Debug: do not byte-swap memory accesses"),
                     cl::init(false));
cl::opt<bool> NoAbiPin("no-abi-pin",
                       cl::desc("Debug: do not pin internal functions to the C ABI"),
                       cl::init(false));

[[noreturn]] void fatal(const Twine &Msg) {
  errs() << "gwtool: error: " << InputFilename << ": " << Msg << "\n";
  exit(1);
}

std::string typeStr(Type *T) {
  std::string S;
  raw_string_ostream OS(S);
  T->print(OS);
  return S;
}

std::string valStr(const Value *V) {
  std::string S;
  raw_string_ostream OS(S);
  V->print(OS);
  return S;
}

// ---------------------------------------------------------------------------
// Layout verification: the frontend laid out every aggregate for PowerPC. After switching the
// data layout to x86, every size, alignment and field offset must be unchanged.
// ---------------------------------------------------------------------------
class LayoutChecker {
public:
  LayoutChecker(const DataLayout &OldDL, const DataLayout &NewDL) : Old(OldDL), New(NewDL) {}

  void check(Type *T, const Twine &Where) {
    if (!T->isSized() || !Seen.insert(T).second)
      return;
    if (Old.getTypeAllocSize(T) != New.getTypeAllocSize(T) ||
        Old.getABITypeAlign(T) != New.getABITypeAlign(T))
      fatal("layout mismatch for " + typeStr(T) + " (" + Where + ")");
    if (auto *ST = dyn_cast<StructType>(T)) {
      const StructLayout *A = Old.getStructLayout(ST);
      const StructLayout *B = New.getStructLayout(ST);
      for (unsigned I = 0, E = ST->getNumElements(); I != E; ++I) {
        if (A->getElementOffset(I) != B->getElementOffset(I))
          fatal("field offset mismatch in " + typeStr(ST) + " (" + Where + ")");
        check(ST->getElementType(I), Where);
      }
    } else if (auto *AT = dyn_cast<ArrayType>(T)) {
      check(AT->getElementType(), Where);
    } else if (auto *VT = dyn_cast<VectorType>(T)) {
      check(VT->getElementType(), Where);
    }
  }

private:
  const DataLayout &Old;
  const DataLayout &New;
  DenseSet<Type *> Seen;
};

// ---------------------------------------------------------------------------
// Memory access byte-swapping
// ---------------------------------------------------------------------------

// Integer type with the same size as T (a scalar or a vector of scalars), or nullptr when
// T occupies a single byte and needs no swap.
Type *swapIntType(Type *T, const DataLayout &DL) {
  if (auto *VT = dyn_cast<FixedVectorType>(T)) {
    Type *E = swapIntType(VT->getElementType(), DL);
    return E ? FixedVectorType::get(E, VT->getNumElements()) : nullptr;
  }
  if (!(T->isIntegerTy() || T->isPointerTy() || T->isHalfTy() || T->isBFloatTy() ||
        T->isFloatTy() || T->isDoubleTy()))
    fatal("cannot byte-swap memory access of type " + typeStr(T));
  uint64_t Bits = DL.getTypeSizeInBits(T);
  if (Bits <= 8)
    return nullptr;
  if (Bits % 8 != 0)
    fatal("non byte-sized memory access of type " + typeStr(T));
  return IntegerType::get(T->getContext(), static_cast<unsigned>(Bits));
}

Value *bswapInt(IRBuilder<> &B, Value *V) {
  auto *IT = cast<IntegerType>(V->getType()->getScalarType());
  unsigned Bits = IT->getBitWidth();
  if (Bits % 16 == 0)
    return B.CreateUnaryIntrinsic(Intrinsic::bswap, V);
  // Odd byte count (i24, i40, ...): widen by one byte, swap, shift the pad byte out.
  Type *WideT = IntegerType::get(V->getContext(), Bits + 8);
  if (auto *VT = dyn_cast<FixedVectorType>(V->getType()))
    WideT = FixedVectorType::get(WideT, VT->getNumElements());
  Value *W = B.CreateZExt(V, WideT);
  W = B.CreateUnaryIntrinsic(Intrinsic::bswap, W);
  W = B.CreateLShr(W, ConstantInt::get(WideT, 8));
  return B.CreateTrunc(W, V->getType());
}

Value *toInt(IRBuilder<> &B, Value *V, Type *IntT) {
  Type *T = V->getType();
  if (T == IntT)
    return V;
  if (T->isPtrOrPtrVectorTy())
    return B.CreatePtrToInt(V, IntT);
  return B.CreateBitCast(V, IntT);
}

Value *fromInt(IRBuilder<> &B, Value *V, Type *T) {
  if (V->getType() == T)
    return V;
  if (T->isPtrOrPtrVectorTy())
    return B.CreateIntToPtr(V, T);
  return B.CreateBitCast(V, T);
}

void copyAccessMetadata(Instruction *From, Instruction *To) {
  // Deliberately not copied: !range, !nonnull, !align, !dereferenceable, !noundef — they
  // describe the logical value, not the swapped bits in memory.
  for (unsigned Kind : {LLVMContext::MD_nontemporal, LLVMContext::MD_access_group,
                        LLVMContext::MD_mem_parallel_loop_access, LLVMContext::MD_tbaa,
                        LLVMContext::MD_alias_scope, LLVMContext::MD_noalias})
    if (MDNode *MD = From->getMetadata(Kind))
      To->setMetadata(Kind, MD);
}

unsigned swapMemoryAccesses(Function &F, const DataLayout &DL, LayoutChecker &LC) {
  SmallVector<Instruction *, 128> Work;
  for (Instruction &I : instructions(F)) {
    if (auto *AI = dyn_cast<AllocaInst>(&I))
      LC.check(AI->getAllocatedType(), F.getName());
    else if (auto *GEP = dyn_cast<GetElementPtrInst>(&I))
      LC.check(GEP->getSourceElementType(), F.getName());
    if (isa<LoadInst>(I) || isa<StoreInst>(I) || isa<AtomicRMWInst>(I) ||
        isa<AtomicCmpXchgInst>(I) || isa<VAArgInst>(I))
      Work.push_back(&I);
  }

  unsigned Count = 0;
  for (Instruction *I : Work) {
    if (auto *LI = dyn_cast<LoadInst>(I)) {
      Type *T = LI->getType();
      if (T->isAggregateType())
        fatal("aggregate load in " + F.getName() + ": " + valStr(LI));
      Type *IT = swapIntType(T, DL);
      if (!IT)
        continue;
      IRBuilder<> B(LI);
      B.SetCurrentDebugLocation(LI->getDebugLoc());
      LoadInst *Raw = B.CreateAlignedLoad(IT, LI->getPointerOperand(), LI->getAlign(),
                                          LI->isVolatile(), LI->getName() + ".be");
      Raw->setAtomic(LI->getOrdering(), LI->getSyncScopeID());
      copyAccessMetadata(LI, Raw);
      Value *V = fromInt(B, bswapInt(B, Raw), T);
      V->takeName(LI);
      LI->replaceAllUsesWith(V);
      LI->eraseFromParent();
      ++Count;
    } else if (auto *SI = dyn_cast<StoreInst>(I)) {
      Value *Val = SI->getValueOperand();
      Type *T = Val->getType();
      if (T->isAggregateType())
        fatal("aggregate store in " + F.getName() + ": " + valStr(SI));
      Type *IT = swapIntType(T, DL);
      if (!IT)
        continue;
      IRBuilder<> B(SI);
      B.SetCurrentDebugLocation(SI->getDebugLoc());
      Value *S = bswapInt(B, toInt(B, Val, IT));
      StoreInst *N =
          B.CreateAlignedStore(S, SI->getPointerOperand(), SI->getAlign(), SI->isVolatile());
      N->setAtomic(SI->getOrdering(), SI->getSyncScopeID());
      copyAccessMetadata(SI, N);
      SI->eraseFromParent();
      ++Count;
    } else if (isa<VAArgInst>(I)) {
      fatal("va_arg instruction in " + F.getName() + " (game code must use the port stdarg.h)");
    } else {
      Type *T = isa<AtomicRMWInst>(I) ? cast<AtomicRMWInst>(I)->getValOperand()->getType()
                                      : cast<AtomicCmpXchgInst>(I)->getNewValOperand()->getType();
      if (swapIntType(T, DL))
        fatal("multi-byte atomic read-modify-write in " + F.getName());
    }
  }
  return Count;
}

// ---------------------------------------------------------------------------
// PowerPC-compatible arithmetic: no traps, no poison where Gekko has defined behavior.
// ---------------------------------------------------------------------------
void makeArithmeticSafe(Function &F) {
  SmallVector<Instruction *, 32> Work;
  for (Instruction &I : instructions(F)) {
    switch (I.getOpcode()) {
    case Instruction::SDiv:
    case Instruction::UDiv:
    case Instruction::SRem:
    case Instruction::URem:
    case Instruction::Shl:
    case Instruction::LShr:
    case Instruction::AShr:
      if (I.getType()->isIntegerTy())
        Work.push_back(&I);
      break;
    case Instruction::FPToSI:
    case Instruction::FPToUI:
      if (I.getType()->isIntegerTy() && I.getOperand(0)->getType()->isFloatingPointTy())
        Work.push_back(&I);
      break;
    default:
      break;
    }
  }

  for (Instruction *I : Work) {
    IRBuilder<> B(I);
    B.SetCurrentDebugLocation(I->getDebugLoc());
    Value *R = nullptr;
    unsigned Op = I->getOpcode();
    auto *T = cast<IntegerType>(I->getType());
    unsigned W = T->getBitWidth();

    if (Op == Instruction::SDiv || Op == Instruction::UDiv || Op == Instruction::SRem ||
        Op == Instruction::URem) {
      Value *A = I->getOperand(0), *D = I->getOperand(1);
      bool Signed = Op == Instruction::SDiv || Op == Instruction::SRem;
      if (auto *C = dyn_cast<ConstantInt>(D))
        if (!C->isZero() && !(Signed && C->isMinusOne()))
          continue;
      Constant *Zero = ConstantInt::get(T, 0);
      Constant *One = ConstantInt::get(T, 1);
      Constant *AllOnes = ConstantInt::getAllOnesValue(T);
      Value *DZero = B.CreateICmpEQ(D, Zero);
      Value *Bad = DZero;
      if (Signed)
        Bad = B.CreateOr(Bad,
                         B.CreateAnd(B.CreateICmpEQ(A, ConstantInt::get(
                                                           T, APInt::getSignedMinValue(W))),
                                     B.CreateICmpEQ(D, AllOnes)));
      Value *SafeD = B.CreateSelect(Bad, One, D);
      Value *Res = B.CreateBinOp(static_cast<Instruction::BinaryOps>(Op), A, SafeD);
      Value *Fallback;
      switch (Op) {
      case Instruction::UDiv:
        Fallback = Zero;
        break;
      case Instruction::URem:
        Fallback = A; // a - 0*d
        break;
      case Instruction::SDiv:
        Fallback = B.CreateSelect(B.CreateICmpSLT(A, Zero), AllOnes, Zero);
        break;
      default: // SRem: x % 0 -> x, INT_MIN % -1 -> 0
        Fallback = B.CreateSelect(DZero, A, Zero);
        break;
      }
      R = B.CreateSelect(Bad, Fallback, Res);
    } else if (Op == Instruction::Shl || Op == Instruction::LShr || Op == Instruction::AShr) {
      Value *Amt = I->getOperand(1);
      if (isa<Constant>(Amt) || (W != 32 && W != 64))
        continue;
      Value *Big = B.CreateICmpUGE(Amt, ConstantInt::get(T, W));
      Value *SafeAmt = B.CreateAnd(Amt, ConstantInt::get(T, W - 1));
      Value *Res =
          B.CreateBinOp(static_cast<Instruction::BinaryOps>(Op), I->getOperand(0), SafeAmt);
      Value *Fallback = Op == Instruction::AShr
                            ? B.CreateAShr(I->getOperand(0), ConstantInt::get(T, W - 1))
                            : static_cast<Value *>(ConstantInt::get(T, 0));
      R = B.CreateSelect(Big, Fallback, Res);
    } else { // FPToSI / FPToUI
      Value *Src = I->getOperand(0);
      Type *I32 = Type::getInt32Ty(I->getContext());
      if (W < 32) {
        // fctiwz (signed 32-bit saturate) then truncate, for both signed and unsigned targets.
        Value *S = B.CreateIntrinsic(Intrinsic::fptosi_sat, {I32, Src->getType()}, {Src});
        R = B.CreateTrunc(S, T);
      } else {
        Intrinsic::ID IID =
            Op == Instruction::FPToSI ? Intrinsic::fptosi_sat : Intrinsic::fptoui_sat;
        R = B.CreateIntrinsic(IID, {T, Src->getType()}, {Src});
      }
    }

    if (!R)
      continue;
    R->takeName(I);
    I->replaceAllUsesWith(R);
    I->eraseFromParent();
  }
}

// ---------------------------------------------------------------------------
// Global initializers
// ---------------------------------------------------------------------------
class GlobalSwapper {
public:
  GlobalSwapper(Module &Mod, const DataLayout &Layout, LayoutChecker &Checker)
      : M(Mod), DL(Layout), LC(Checker) {}

  void run() {
    for (GlobalVariable &GV : M.globals()) {
      if (GV.getName().starts_with("llvm."))
        continue;
      LC.check(GV.getValueType(), GV.getName());
      if (!GV.hasInitializer())
        continue;
      Cur = &GV;
      size_t Before = Fixups.size();
      Constant *Init = GV.getInitializer();
      Constant *NewInit = swap(Init, 0);
      if (NewInit != Init)
        GV.setInitializer(NewInit);
      if (Fixups.size() != Before) {
        GV.setConstant(false);
        GV.setExternallyInitialized(true);
        GV.setUnnamedAddr(GlobalValue::UnnamedAddr::None);
      }
    }
    emitTable();
  }

  size_t numFixups() const { return Fixups.size(); }

private:
  Constant *swap(Constant *C, uint64_t Off) {
    Type *T = C->getType();
    if (isa<ConstantAggregateZero>(C) || isa<UndefValue>(C) || isa<ConstantPointerNull>(C))
      return C;
    if (auto *CI = dyn_cast<ConstantInt>(C)) {
      unsigned W = CI->getBitWidth();
      if (W <= 8)
        return C;
      if (W % 8 != 0)
        fatal("odd-width integer in initializer of " + Cur->getName());
      return ConstantInt::get(T, CI->getValue().byteSwap());
    }
    if (auto *CF = dyn_cast<ConstantFP>(C)) {
      APInt Bits = CF->getValueAPF().bitcastToAPInt();
      if (Bits.getBitWidth() <= 8)
        return C;
      if (!(T->isHalfTy() || T->isBFloatTy() || T->isFloatTy() || T->isDoubleTy()))
        fatal("unsupported floating type " + typeStr(T) + " in initializer of " +
              Cur->getName());
      return ConstantFP::get(T->getContext(), APFloat(T->getFltSemantics(), Bits.byteSwap()));
    }
    if (auto *CDS = dyn_cast<ConstantDataSequential>(C)) {
      Type *ET = CDS->getElementType();
      if (DL.getTypeStoreSize(ET) <= 1)
        return C;
      uint64_t ES = DL.getTypeAllocSize(ET);
      SmallVector<Constant *, 32> Elts;
      for (unsigned I = 0, E = CDS->getNumElements(); I != E; ++I)
        Elts.push_back(swap(CDS->getElementAsConstant(I), Off + I * ES));
      if (auto *AT = dyn_cast<ArrayType>(T))
        return ConstantArray::get(AT, Elts);
      return ConstantVector::get(Elts);
    }
    if (auto *CA = dyn_cast<ConstantArray>(C)) {
      uint64_t ES = DL.getTypeAllocSize(T->getArrayElementType());
      SmallVector<Constant *, 32> Elts;
      for (unsigned I = 0, E = CA->getNumOperands(); I != E; ++I)
        Elts.push_back(swap(CA->getOperand(I), Off + I * ES));
      return ConstantArray::get(cast<ArrayType>(T), Elts);
    }
    if (auto *CS = dyn_cast<ConstantStruct>(C)) {
      auto *ST = cast<StructType>(T);
      LC.check(ST, Cur->getName());
      const StructLayout *SL = DL.getStructLayout(ST);
      SmallVector<Constant *, 32> Elts;
      for (unsigned I = 0, E = CS->getNumOperands(); I != E; ++I)
        Elts.push_back(swap(CS->getOperand(I), Off + SL->getElementOffset(I)));
      return ConstantStruct::get(ST, Elts);
    }
    if (auto *CV = dyn_cast<ConstantVector>(C)) {
      uint64_t ES = DL.getTypeAllocSize(CV->getType()->getElementType());
      SmallVector<Constant *, 16> Elts;
      for (unsigned I = 0, E = CV->getNumOperands(); I != E; ++I)
        Elts.push_back(swap(CV->getOperand(I), Off + I * ES));
      return ConstantVector::get(Elts);
    }
    // Link-time values: addresses of globals/functions, blockaddress, constant expressions.
    if (T->isPointerTy() || T->isIntegerTy()) {
      uint64_t Size = DL.getTypeStoreSize(T);
      if (Size == 4) {
        Fixups.emplace_back(Cur, Off);
        return C;
      }
      if (Size == 1)
        return C;
    }
    fatal("unsupported constant in initializer of " + Cur->getName() + ": " + valStr(C));
  }

  void emitTable() {
    if (Fixups.empty())
      return;
    LLVMContext &Ctx = M.getContext();
    Type *I8 = Type::getInt8Ty(Ctx);
    Type *I32 = Type::getInt32Ty(Ctx);
    PointerType *P = PointerType::getUnqual(Ctx);
    SmallVector<Constant *, 64> Entries;
    for (auto &[GV, Off] : Fixups) {
      Constant *Idx[] = {ConstantInt::get(I32, Off)};
      Entries.push_back(ConstantExpr::getGetElementPtr(I8, GV, ArrayRef<Constant *>(Idx)));
    }
    auto *AT = ArrayType::get(P, Entries.size());
    auto *Table = new GlobalVariable(M, AT, /*isConstant=*/true, GlobalValue::InternalLinkage,
                                     ConstantArray::get(AT, Entries), "__gw_fixups");
    Table->setSection(".gwfix$m");
    Table->setAlignment(Align(4));
    appendToUsed(M, {Table});
  }

  Module &M;
  const DataLayout &DL;
  LayoutChecker &LC;
  GlobalVariable *Cur = nullptr;
  std::vector<std::pair<GlobalVariable *, uint64_t>> Fixups;
};

// ---------------------------------------------------------------------------
// The bridge's ABI pin
// ---------------------------------------------------------------------------
// The port's guest->native bridge (pc/platform/gw_ppc.c) calls engine functions BY ADDRESS. It
// resolves a guest PPC address to a native one through melee-pc.map and calls it as cdecl,
// through gw_ppc_native_fn. LLVM cannot see those call sites: to it, a function that is `static`
// in its TU and only ever called directly has no other callers in the universe, so every
// interprocedural pass is free to rewrite its interface.
//
// And they do. GlobalOpt retargets such a function to `fastcc` - on i686 the first two integer
// arguments in ECX and EDX, floats in XMM0-2 - while the bridge is still pushing them on the
// stack. DeadArgumentElimination deletes arguments it believes nobody passes, shifting the rest
// down. ArgumentPromotion turns a pointer parameter into loaded values. IPSCCP replaces a
// parameter with the constant every VISIBLE caller happens to pass.
//
// Every one of those is silent and bizarre at the far end rather than loud. grTSeak_80223908
// was the first found: its prologue became `mov edi, ecx`, so all three of a custom stage's
// map-gobj calls arrived with map_id == 0 and Meta Crystal rendered black. An audit of the
// linked exe (tools/mex_port/audit_bridge_abi.py) then found 294 more internal functions that
// the bridge can reach and that read an argument register the bridge never sets.
//
// So tell LLVM the truth instead of patching the symptoms one function at a time: the address
// of every internal function IS taken, from outside this module. One external, address-taking
// global per TU says exactly that, and every pass above checks precisely that property
// (`hasAddressTaken`) before it rewrites an interface. It is not a heuristic and it cannot
// misclassify: no internal function can acquire a private convention, whether or not anything
// bridges to it today.
//
// Cost: the internal functions keep the platform C ABI and stay alive in the image. Inlining is
// unaffected (inlining an internal function does not change the out-of-line copy's interface),
// and the pointer array itself is a few bytes per TU.
//
// It is built AFTER GlobalSwapper has run, deliberately. The swapper rewrites pointer-sized
// link-time values in global initializers into the __gw_fixups table because game memory is
// big-endian; this array is host data that game code never reads, and must not be swapped.
GlobalVariable *pinInternalAbi(Module &M) {
  LLVMContext &Ctx = M.getContext();
  PointerType *P = PointerType::getUnqual(Ctx);
  SmallVector<Constant *, 64> Pins;
  for (Function &F : M) {
    if (F.isDeclaration() || F.isIntrinsic() || !F.hasLocalLinkage())
      continue;
    Pins.push_back(&F);
  }
  if (Pins.empty())
    return nullptr;
  // One symbol per TU, named after the TU, so two objects can never collide.
  std::string Tag = M.getSourceFileName();
  if (Tag.empty())
    Tag = M.getModuleIdentifier();
  for (char &C : Tag)
    if (!((C >= '0' && C <= '9') || (C >= 'A' && C <= 'Z') || (C >= 'a' && C <= 'z')))
      C = '_';
  auto *AT = ArrayType::get(P, Pins.size());
  auto *GV = new GlobalVariable(M, AT, /*isConstant=*/true, GlobalValue::ExternalLinkage,
                                ConstantArray::get(AT, Pins), "__gw_abi_pin_" + Tag);
  GV->setAlignment(Align(4));
  return GV;
}

// The pin's invariant, checked rather than assumed: after the pipeline, every function this
// module defines still has the calling convention and the exact signature it was compiled with.
//
// This is what makes the pin trustworthy. It is an exact comparison, not a guess about
// prologues, and it fires inside the compiler - at the moment and in the TU where a pass would
// have rewritten an interface - rather than months later as a wrong argument in a bridged call.
// If LLVM grows a pass that rewrites an interface some other way, or someone passes
// --no-abi-pin, this fails the TU and says so by name.
using AbiSnapshot = StringMap<std::pair<FunctionType *, CallingConv::ID>>;

AbiSnapshot snapshotAbi(Module &M) {
  AbiSnapshot S;
  for (Function &F : M)
    if (!F.isDeclaration() && !F.isIntrinsic())
      S[F.getName()] = {F.getFunctionType(), F.getCallingConv()};
  return S;
}

void checkAbiUnchanged(Module &M, const AbiSnapshot &Before) {
  for (Function &F : M) {
    if (F.isDeclaration() || F.isIntrinsic())
      continue;
    auto It = Before.find(F.getName());
    if (It == Before.end())
      continue; // a pass invented it; nothing outside this module can call it by address
    if (F.getCallingConv() != It->second.second)
      fatal("optimization gave " + F.getName() +
            " a private calling convention; the guest->native bridge calls it as cdecl "
            "(see pinInternalAbi)");
    if (F.getFunctionType() != It->second.first)
      fatal("optimization rewrote the signature of " + F.getName() +
            "; the guest->native bridge calls it with its original one (see pinInternalAbi)");
  }
}

// Drop the pin once every interprocedural pass has run, and let GlobalDCE reclaim the internal
// functions that were inlined away everywhere - the pin kept them alive, and without this a
// typical TU's object grew about 20%. Nothing can rewrite an interface after this point: the
// optimization pipeline is over and only codegen follows, so the functions that survive keep the
// C ABI the pin won them.
void unpinInternalAbi(Module &M, GlobalVariable *Pin) {
  if (Pin == nullptr)
    return;
  Pin->eraseFromParent();
  ModuleAnalysisManager MAM;
  PassBuilder PB;
  PB.registerModuleAnalyses(MAM);
  ModulePassManager MPM;
  MPM.addPass(GlobalDCEPass());
  MPM.run(M, MAM);
}

// ---------------------------------------------------------------------------
// Symbols, attributes, target details
// ---------------------------------------------------------------------------
void renameSymbols(Module &M) {
  StringSet<> Renamed;
  for (GlobalValue &GV : M.global_values()) {
    if (GV.hasLocalLinkage())
      continue;
    StringRef N = GV.getName();
    if (N.empty() || N.starts_with("llvm.") || N.starts_with("__gwrt_"))
      continue;
    std::string Old = N.str();
    std::string New = SymPrefix + Old;
    GV.setName(New);
    if (GV.getName() != New)
      fatal("symbol clash while renaming " + Old + " -> " + New);
    Renamed.insert(Old);
  }
  // COFF: comdat keys must follow their renamed leader.
  for (GlobalObject &GO : M.global_objects()) {
    Comdat *C = GO.getComdat();
    if (!C || !Renamed.contains(C->getName()))
      continue;
    Comdat *NC = M.getOrInsertComdat(SymPrefix + C->getName().str());
    NC->setSelectionKind(C->getSelectionKind());
    GO.setComdat(NC);
  }
}

void checkForbidden(Module &M) {
  for (Function &F : M) {
    if (F.getName().starts_with("llvm.ppc."))
      fatal("PowerPC intrinsic " + F.getName() + " used (replace in source)");
    for (Instruction &I : instructions(F)) {
      if (auto *CB = dyn_cast<CallBase>(&I))
        if (CB->isInlineAsm())
          fatal("inline asm in " + F.getName() + " (replace in source)");
      for (Value *Op : I.operands())
        if (Op->getType()->isPPC_FP128Ty() || Op->getType()->isX86_FP80Ty())
          fatal("long double used in " + F.getName() + " (compile with -mlong-double-64)");
    }
  }
}

void fixAttributes(Module &M) {
  for (Function &F : M) {
    if (F.isIntrinsic())
      continue;
    F.removeFnAttr("target-cpu");
    F.removeFnAttr("target-features");
    F.removeFnAttr("tune-cpu");
    F.setSection("");
    if (F.isDeclaration()) {
      // Native (MSVC) callees do not extend narrow return values.
      F.removeRetAttr(Attribute::ZExt);
      F.removeRetAttr(Attribute::SExt);
    } else {
      F.addFnAttr("target-cpu", CPUName);
      F.addFnAttr("target-features", FeatureStr);
      // Native (MSVC) callers do not extend narrow arguments.
      for (Argument &A : F.args()) {
        A.removeAttr(Attribute::ZExt);
        A.removeAttr(Attribute::SExt);
      }
    }
    for (Instruction &I : instructions(F))
      if (auto *CB = dyn_cast<CallBase>(&I)) {
        Function *Callee = CB->getCalledFunction();
        if (!Callee || Callee->isDeclaration()) {
          CB->removeRetAttr(Attribute::ZExt);
          CB->removeRetAttr(Attribute::SExt);
        }
      }
  }
  for (GlobalVariable &GV : M.globals()) {
    if (GV.getName().starts_with("llvm."))
      continue;
    GV.setSection("");
    // Game globals inherit the GameCube's 32-byte DMA alignment. On hardware the DOL's data
    // layout supplied it and the SDK relies on it: ARQ/DVD transfers assert that both endpoints
    // are 32-aligned (devcom.c:420-423), and DCFlushRange/DCInvalidateRange operate on 32-byte
    // cache lines. A host linker only honours the type's natural alignment, so a global handed to
    // a transfer -- e.g. hsd_SynthSFXLoadBuf at synth.c:190 -- lands wherever 4-byte alignment
    // puts it and trips the assert. Widening every definition is a few hundred KB of padding and
    // removes the whole failure class; declarations are left alone so they do not conflict.
    if (!GV.isDeclaration() && GV.getAlign().valueOrOne() < Align(32))
      GV.setAlignment(Align(32));
  }
}

void fixModuleFlags(Module &M) {
  if (NamedMDNode *NMD = M.getModuleFlagsMetadata()) {
    SmallVector<MDNode *, 8> Keep;
    for (MDNode *Op : NMD->operands()) {
      auto *Key = Op->getNumOperands() > 1 ? dyn_cast<MDString>(Op->getOperand(1)) : nullptr;
      if (Key && (Key->getString() == "Dwarf Version" || Key->getString() == "target-abi"))
        continue;
      Keep.push_back(Op);
    }
    NMD->clearOperands();
    for (MDNode *K : Keep)
      NMD->addOperand(K);
  }
  if (!M.getModuleFlag("CodeView"))
    M.addModuleFlag(Module::Warning, "CodeView", 1);
}

void writeImports(Module &M) {
  if (ImportsFile.empty())
    return;
  std::error_code EC;
  raw_fd_ostream OS(ImportsFile, EC, sys::fs::OF_Text);
  if (EC)
    fatal("cannot write " + ImportsFile + ": " + EC.message());
  for (Function &F : M) {
    if (!F.isDeclaration() || F.isIntrinsic() || F.use_empty())
      continue;
    OS << "F\t" << F.getName() << "\t";
    F.getFunctionType()->print(OS);
    if (F.hasParamAttribute(0, Attribute::StructRet))
      OS << "\tsret";
    OS << "\n";
  }
  for (GlobalVariable &GV : M.globals()) {
    if (GV.hasInitializer() || GV.getName().starts_with("llvm.") || GV.use_empty())
      continue;
    OS << "D\t" << GV.getName() << "\t";
    GV.getValueType()->print(OS);
    OS << "\n";
  }
}

} // namespace

int main(int argc, char **argv) {
  InitLLVM X(argc, argv);
  LLVMInitializeX86TargetInfo();
  LLVMInitializeX86Target();
  LLVMInitializeX86TargetMC();
  LLVMInitializeX86AsmPrinter();
  LLVMInitializeX86AsmParser();
  cl::ParseCommandLineOptions(argc, argv, "gwtool: Melee PC port big-endian-memory transformer\n");

  LLVMContext Ctx;
  SMDiagnostic Err;
  std::unique_ptr<Module> M = parseIRFile(InputFilename, Err, Ctx);
  if (!M) {
    Err.print(argv[0], errs());
    return 1;
  }

  Triple OldT(M->getTargetTriple());
  if (OldT.getArch() != Triple::ppc)
    fatal("expected a ppc32 module, got triple " + OldT.str());
  const DataLayout OldDL = M->getDataLayout();
  if (!OldDL.isBigEndian())
    fatal("expected a big-endian data layout");

  Triple NewT("i686-pc-windows-msvc");
  std::string Error;
  const Target *Tgt = TargetRegistry::lookupTarget(NewT, Error);
  if (!Tgt)
    fatal("target lookup failed: " + Error);
  CodeGenOptLevel CGOpt = OptLevelOpt == 0   ? CodeGenOptLevel::None
                          : OptLevelOpt == 1 ? CodeGenOptLevel::Less
                          : OptLevelOpt == 2 ? CodeGenOptLevel::Default
                                             : CodeGenOptLevel::Aggressive;
  TargetOptions TOpts;
  std::unique_ptr<TargetMachine> TM(Tgt->createTargetMachine(
      NewT, CPUName, FeatureStr, TOpts, Reloc::Static, std::nullopt, CGOpt));
  if (!TM)
    fatal("could not create target machine");
  const DataLayout NewDL = TM->createDataLayout();

  checkForbidden(*M);
  LayoutChecker LC(OldDL, NewDL);
  for (StructType *ST : M->getIdentifiedStructTypes())
    if (!ST->isOpaque())
      LC.check(ST, ST->getName());

  M->setTargetTriple(NewT);
  M->setDataLayout(NewDL);

  renameSymbols(*M);
  fixAttributes(*M);
  fixModuleFlags(*M);

  GlobalSwapper GS(*M, NewDL, LC);
  if (!NoSwap)
    GS.run();
  for (Function &F : *M) {
    if (F.isDeclaration())
      continue;
    if (!NoSafeArith)
      makeArithmeticSafe(F);
    if (!NoSwap)
      swapMemoryAccesses(F, NewDL, LC);
  }

  GlobalVariable *AbiPin = NoAbiPin ? nullptr : pinInternalAbi(*M);
  AbiSnapshot AbiBefore = snapshotAbi(*M);

  if (verifyModule(*M, &errs()))
    fatal("module verification failed after transform");

  if (OptLevelOpt > 0) {
    LoopAnalysisManager LAM;
    FunctionAnalysisManager FAM;
    CGSCCAnalysisManager CGAM;
    ModuleAnalysisManager MAM;
    PassBuilder PB(TM.get());
    PB.registerModuleAnalyses(MAM);
    PB.registerCGSCCAnalyses(CGAM);
    PB.registerFunctionAnalyses(FAM);
    PB.registerLoopAnalyses(LAM);
    PB.crossRegisterProxies(LAM, FAM, CGAM, MAM);
    OptimizationLevel OL = OptLevelOpt == 1   ? OptimizationLevel::O1
                           : OptLevelOpt == 2 ? OptimizationLevel::O2
                                              : OptimizationLevel::O3;
    ModulePassManager MPM = PB.buildPerModuleDefaultPipeline(OL);
    MPM.run(*M, MAM);
  }
  if (AbiPin != nullptr)
    checkAbiUnchanged(*M, AbiBefore);
  unpinInternalAbi(*M, AbiPin);

  writeImports(*M);

  std::error_code EC;
  bool Text = EmitKind == "ll" || EmitKind == "asm";
  ToolOutputFile Out(OutputFilename, EC, Text ? sys::fs::OF_Text : sys::fs::OF_None);
  if (EC)
    fatal("cannot open output: " + EC.message());

  if (EmitKind == "ll") {
    M->print(Out.os(), nullptr);
  } else if (EmitKind == "bc") {
    WriteBitcodeToFile(*M, Out.os());
  } else if (EmitKind == "obj" || EmitKind == "asm") {
    legacy::PassManager CGPM;
    if (TM->addPassesToEmitFile(CGPM, Out.os(), nullptr,
                                EmitKind == "obj" ? CodeGenFileType::ObjectFile
                                                  : CodeGenFileType::AssemblyFile))
      fatal("target cannot emit this file type");
    CGPM.run(*M);
  } else {
    fatal("unknown --emit kind " + EmitKind);
  }
  Out.keep();
  return 0;
}
