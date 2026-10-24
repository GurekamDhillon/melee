//! Migration aid: replace c2rust's duplicated records with generated melee_types.
//! The output remains review input, not an equivalence proof.
use std::collections::{BTreeMap, HashMap};
use quote::{quote, ToTokens};
use syn::{visit_mut::{self, VisitMut}, *};

fn ts<T: ToTokens>(t: &T) -> String { t.to_token_stream().to_string() }
fn name(t: &Type) -> Option<String> {
    match t { Type::Path(p) => p.path.segments.last().map(|s|s.ident.to_string()), _=>None }
}
fn basename(t: &Type) -> Option<String> {
    match t { Type::Array(a)=>basename(&a.elem), Type::Ptr(p)=>basename(&p.elem), Type::Reference(r)=>basename(&r.elem), _=>name(t) }
}
fn fields(f: &File) -> HashMap<String, Vec<Field>> {
    f.items.iter().filter_map(|i|match i {
        Item::Struct(s)=>Some((s.ident.to_string(),s.fields.iter().cloned().collect())),
        Item::Union(s)=>Some((s.ident.to_string(),s.fields.named.iter().cloned().collect())), _=>None
    }).collect()
}
fn aliases(f:&File)->HashMap<String,Type> { f.items.iter().filter_map(|i|match i{Item::Type(t)=>Some((t.ident.to_string(),(*t.ty).clone())),_=>None}).collect() }
fn canon(n:&str,a:&HashMap<String,Type>)->String {
    let mut s=n.to_string(); for _ in 0..20 { if let Some(t)=a.get(&s) { if let Some(n)=name(t){if n==s{break} s=n;continue} } break } s
}
fn fld(f:&Field)->String { f.ident.as_ref().unwrap().to_string() }
fn clean(n:&str)->String {
    let n=n.strip_prefix("r#").unwrap_or(n);
    if n.starts_with("c2rust_unnamed") || n.starts_with("anon_") { "@anon".into() }
    else if n.starts_with("bf_") || n.len()>100 { "@bits".into() }
    else if let Some(n)=n.strip_suffix("_0"){n.into()}else if n=="self_"{"self".into()}else{n.into()}
}
fn signature(v:&[Field])->Vec<String>{v.iter().filter(|f|!fld(f).starts_with("_bitfield_align")).map(|f|clean(&fld(f))).collect()}
fn primsig(t:&Type,a:&HashMap<String,Type>)->String {
    match t {
        Type::Ptr(p)=>format!("*{}",primsig(&p.elem,a)),Type::Array(p)=>format!("[{};{}]",primsig(&p.elem,a),ts(&p.len)),
        _=>{let n=canon(&name(t).unwrap_or_default(),a); match n.as_str(){"c_int"=>"i32","c_uint"=>"u32","c_float"=>"f32","c_double"=>"f64","c_char"=>"i8","c_uchar"=>"u8","c_short"=>"i16","c_ushort"=>"u16","c_long"=>"i32","c_ulong"=>"u32",_=>&n}.into()}
    }
}
fn zero(e:&Expr,z:&HashMap<String,Expr>)->bool { match e {
    Expr::Lit(l)=>match &l.lit{Lit::Int(i)=>i.base10_parse::<u64>().ok()==Some(0),Lit::Float(f)=>f.base10_parse::<f64>().ok()==Some(0.0),_=>false},
    Expr::Cast(c)=>zero(&c.expr,z), Expr::Paren(p)=>zero(&p.expr,z),
    Expr::Repeat(r)=>zero(&r.expr,z), Expr::Array(a)=>a.elems.iter().all(|e|zero(e,z)),
    Expr::Struct(s)=>s.fields.iter().all(|f|zero(&f.expr,z)),
    Expr::Path(p)=>p.path.is_ident("None")||p.path.get_ident().and_then(|i|z.get(&i.to_string())).is_some_and(|e|zero(e,z)),
    Expr::Call(c)=>ts(&c.func).contains(":: ptr :: null") && c.args.is_empty(), _=>false
} }

struct Fix {
    records:HashMap<String,Vec<Field>>, aliases:HashMap<String,Type>, map:HashMap<String,String>,
    fieldmap:HashMap<(String,String),String>, vars:HashMap<String,Type>, globals:HashMap<String,Type>,
    returns:HashMap<String,Type>, va:HashMap<String,usize>, zeros:HashMap<String,Expr>, fnmap:HashMap<String,String>, current:String, fused:BTreeMap<String,(usize,usize)>,
}
impl Fix {
    fn resolve(&self,t:Type)->Type { if let Some(n)=name(&t){if let Some(a)=self.aliases.get(&n){if ts(a)!=ts(&t){return self.resolve(a.clone())}}}t }
    fn ty(&self,e:&Expr)->Option<Type>{ match e {
        Expr::Path(p)=>{let n=p.path.segments.last()?.ident.to_string();self.vars.get(&n).or_else(||self.globals.get(&n)).cloned()},
        Expr::Paren(p)=>self.ty(&p.expr),Expr::Group(p)=>self.ty(&p.expr),Expr::Cast(c)=>Some((*c.ty).clone()),
        Expr::Reference(r)=>{let t=self.ty(&r.expr)?;Some(parse_quote!(*mut #t))},
        Expr::Unary(u)=>match u.op{UnOp::Deref(_)=>match self.resolve(self.ty(&u.expr)?){Type::Ptr(p)=>Some(*p.elem),Type::Reference(p)=>Some(*p.elem),_=>None},_=>self.ty(&u.expr)},
        Expr::Field(f)=>{let t=self.resolve(self.ty(&f.base)?);let n=name(&t)?;let m=ts(&f.member);self.records.get(&n)?.iter().find(|x|fld(x)==m).map(|x|x.ty.clone())},
        Expr::Index(i)=>match self.resolve(self.ty(&i.expr)?){Type::Array(a)=>Some(*a.elem),Type::Ptr(p)=>Some(*p.elem),_=>None},
        Expr::Call(c)=>{if let Expr::Path(p)=&*c.func{self.returns.get(&p.path.segments.last()?.ident.to_string()).cloned()}else{None}},
        Expr::MethodCall(m)=>{match m.method.to_string().as_str(){"as_mut_ptr"|"as_ptr"=>match self.resolve(self.ty(&m.receiver)?){Type::Array(a)=>{let t=a.elem;Some(parse_quote!(*mut #t))},_=>None},"offset"|"add"|"sub"|"wrapping_offset"|"wrapping_add"|"wrapping_sub"=>self.ty(&m.receiver),_=>None}},
        Expr::Struct(s)=>Some(Type::Path(TypePath{qself:None,path:s.path.clone()})),
        Expr::Binary(b)=>self.ty(&b.left).or_else(||self.ty(&b.right)),
        Expr::Lit(l)=>match &l.lit{Lit::Float(f)=>Some(if f.suffix()=="f64"{parse_quote!(f64)}else{parse_quote!(f32)}),Lit::Int(_)=>Some(parse_quote!(i32)),_=>None},_=>None
    }}
    fn newfield(&self,t:&Type,m:&Member)->Option<Member>{
        let n=canon(&name(t)?,&self.aliases);let old=ts(m);
        self.fieldmap.get(&(n,old)).map(|n|Member::Named(syn::parse_str(n).unwrap()))
    }
}
fn mul(e:&Expr)->Option<(&Expr,&Expr)>{match e{Expr::Paren(p)=>mul(&p.expr),Expr::Cast(c) if ["f32","c_float","f64","c_double"].contains(&name(&c.ty).unwrap_or_default().as_str())=>mul(&c.expr),Expr::Binary(b) if matches!(b.op,BinOp::Mul(_))=>Some((&b.left,&b.right)),_=>None}}
fn fused_expr(b:&ExprBinary,k:&str)->Option<Expr>{
    let compound=matches!(b.op,BinOp::AddAssign(_)|BinOp::SubAssign(_));
    let sub=matches!(b.op,BinOp::Sub(_)|BinOp::SubAssign(_));
    if !matches!(b.op,BinOp::Add(_)|BinOp::Sub(_)|BinOp::AddAssign(_)|BinOp::SubAssign(_)){return None}
    let lhs=&b.left;let rhs=&b.right;
    let (a,x,y)=if !compound {if let Some((a,x))=mul(lhs){let y:Expr=if sub{parse_quote!(-(#rhs))}else{*rhs.clone()};(a.clone(),x.clone(),y)}else if let Some((a,x))=mul(rhs){let a:Expr=if sub{parse_quote!(-(#a))}else{a.clone()};(a,x.clone(),*lhs.clone())}else{return None}}
    else if let Some((a,x))=mul(rhs){let a:Expr=if sub{parse_quote!(-(#a))}else{a.clone()};(a,x.clone(),*lhs.clone())}else{return None};
    let val:Expr=if k=="f32"{parse_quote!(melee_core::fp::fmadds(#a,#x,#y))}else{parse_quote!((#a).mul_add(#x,#y))};
    Some(if compound{parse_quote!(#lhs = #val)}else{val})
}
impl VisitMut for Fix {
    fn visit_item_static_mut(&mut self,s:&mut ItemStatic){self.vars.insert(s.ident.to_string(),(*s.ty).clone());visit_mut::visit_item_static_mut(self,s);}
    fn visit_item_fn_mut(&mut self,f:&mut ItemFn){
        let saved=self.vars.clone();let prev=self.current.clone();self.current=f.sig.ident.to_string();self.vars.clear();
        for a in &f.sig.inputs{if let FnArg::Typed(a)=a{if let Pat::Ident(i)=&*a.pat{self.vars.insert(i.ident.to_string(),(*a.ty).clone());}}}
        f.attrs.retain(|a|!a.path().is_ident("no_mangle")&&!a.path().is_ident("linkage"));
        f.vis=parse_quote!(pub);
        f.attrs.retain(|a|!a.path().is_ident("export_name"));
        if let Some(n)=self.fnmap.get(&f.sig.ident.to_string()){f.sig.ident=syn::parse_str(n).unwrap();}
        visit_mut::visit_item_fn_mut(self,f);self.vars=saved;self.current=prev;
    }
    fn visit_local_mut(&mut self,l:&mut Local){
        if let Pat::Type(p)=&l.pat { if let Pat::Ident(i)=&*p.pat {self.vars.insert(i.ident.to_string(),(*p.ty).clone());}}
        else if let Pat::Ident(i)=&l.pat{if let Some(init)=&l.init{if let Some(t)=self.ty(&init.expr){let t=if i.by_ref.is_some(){parse_quote!(*mut #t)}else{t};self.vars.insert(i.ident.to_string(),t);}}}
        visit_mut::visit_local_mut(self,l);
    }
    fn visit_type_mut(&mut self,t:&mut Type){
        if let Some(n)=name(t){
            if let Some(m)=self.map.get(&n){*t=syn::parse_str(m).unwrap();return}
            if let Some(a)=self.aliases.get(&n).cloned(){if ts(&a)!=ts(t){*t=a;self.visit_type_mut(t);return}}
        }
        visit_mut::visit_type_mut(self,t);
    }
    fn visit_expr_mut(&mut self,e:&mut Expr){
        if let Expr::Struct(s)=e {
            if zero(&Expr::Struct(s.clone()),&self.zeros){let mut t:Type=syn::parse2(s.path.to_token_stream()).unwrap();self.visit_type_mut(&mut t);*e=parse_quote!(unsafe { core::mem::zeroed::<#t>() });return}
        }
        let fm=match e{Expr::Field(f)=>self.ty(&f.base).and_then(|t|self.newfield(&t,&f.member)),_=>None};
        let sm=match e{Expr::Struct(s)=>s.path.segments.last().map(|x|x.ident.to_string()),_=>None};
        let variadic=match e{Expr::Call(c)=>if let Expr::Path(p)=&*c.func{p.path.segments.last().and_then(|s|self.va.get(&s.ident.to_string()).copied())}else{None},_=>None};
        let vatypes=if let Expr::Call(c)=e{c.args.iter().map(|a|self.ty(a)).collect::<Vec<_>>()}else{vec![]};
        let fk=if matches!(e,Expr::Binary(_)){self.ty(e).map(|t|primsig(&t,&self.aliases)).filter(|k|k=="f32"||k=="f64")}else{None};
        visit_mut::visit_expr_mut(self,e);
        if self.current!="ScriptGame_StageSwitch" {if let (Expr::Binary(b),Some(k))=(&mut *e,fk){if let Some(fe)=fused_expr(b,&k){let c=self.fused.entry(self.current.clone()).or_default();if k=="f32"{c.0+=1}else{c.1+=1}*e=fe;}}}
        if let Expr::Path(p)=e {if let Some(id)=p.path.get_ident(){if let Some(n)=self.fnmap.get(&id.to_string()){p.path=syn::parse_str(n).unwrap();}}}
        if let (Expr::Field(f),Some(m))=(&mut *e,fm){f.member=m;}
        if let (Expr::Struct(s),Some(n))=(&mut *e,sm){
            for f in &mut s.fields{if let Some(m)=self.newfield(&syn::parse_str::<Type>(&n).unwrap(),&f.member){f.member=m;}}
            if let Some(m)=self.map.get(&n){s.path=syn::parse_str(m).unwrap();}
        }
        if let (Expr::Call(c),Some(n))=(&mut *e,variadic){
            let a=c.args.iter().skip(n).enumerate().map(|(i,a)|{
                let ty=vatypes.get(i+n).cloned().flatten().map(|t|self.resolve(t));
                match ty{Some(Type::Ptr(_))|Some(Type::Reference(_))=>quote!(crate::oracle::Va::P(#a as *const core::ffi::c_void)),Some(t) if ["f64","c_double"].contains(&name(&t).unwrap_or_default().as_str())=>quote!(crate::oracle::Va::D(#a as f64)),_=>quote!(crate::oracle::Va::I32(#a as i32))}
            }).collect::<Vec<_>>();
            c.args=c.args.iter().take(n).cloned().collect();c.args.push(syn::parse2(quote!(&[#(#a),*])).unwrap());
        }
    }
}

#[derive(Default)]
struct Lift { items:Vec<Item>, names:HashMap<String,String> }
fn linkname(attrs:&[Attribute])->Option<String>{attrs.iter().find_map(|a|if a.path().is_ident("link_name")||a.path().is_ident("export_name"){if let Meta::NameValue(v)=&a.meta{if let Expr::Lit(ExprLit{lit:Lit::Str(s),..})=&v.value{return Some(s.value())}}None}else{None})}
impl VisitMut for Lift {
    fn visit_block_mut(&mut self,b:&mut Block){
        let mut keep=vec![];
        for s in std::mem::take(&mut b.stmts){match s {
            Stmt::Item(Item::ForeignMod(m))=>{for i in &m.items{if let ForeignItem::Fn(f)=i{if let Some(n)=linkname(&f.attrs){self.names.insert(f.sig.ident.to_string(),n);}}}self.items.push(Item::ForeignMod(m));},
            Stmt::Item(Item::Fn(mut f)) if linkname(&f.attrs).is_some()=>{self.names.insert(f.sig.ident.to_string(),linkname(&f.attrs).unwrap());visit_mut::visit_item_fn_mut(self,&mut f);self.items.push(Item::Fn(f));},
            s=>keep.push(s)
        }}
        b.stmts=keep;visit_mut::visit_block_mut(self,b);
    }
}

fn main(){
    let args:Vec<_>=std::env::args().collect();
    let raw=std::fs::read_to_string(&args[1]).unwrap();
    let raw=raw.replace(" _: "," _pad: ").replace("(_: ","(_pad: ").replace("._)","._pad)").replace("._,","._pad,").replace("._ =","._pad =");
    let mut f=syn::parse_file(&raw).unwrap_or_else(|e|panic!("parse {:?}: {e}",e.span().start()));
    let mut lift=Lift::default();lift.visit_file_mut(&mut f);f.items.append(&mut lift.items);
    let gf=syn::parse_file(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
    let rec=fields(&f);let grec=fields(&gf);let al=aliases(&f);let gal=aliases(&gf);
    let mut map:HashMap<String,String>=HashMap::new();
    for n in al.keys(){if gal.contains_key(n){map.insert(n.clone(),n.clone());}}
    for n in rec.keys(){let cn=canon(n,&gal);if grec.contains_key(&cn){map.insert(n.clone(),cn);}}
    for (a,t) in &al{if let Some(n)=name(t){let cn=canon(a,&gal);if rec.contains_key(&n)&&grec.contains_key(&cn){map.insert(n,cn);}}}
    let mut fm=HashMap::new();
    for _ in 0..8{
        for (rn,gn) in map.clone(){
            let (Some(rv),Some(gv))=(rec.get(&rn),grec.get(&gn))else{continue};
            for rf in rv{
                let rfn=fld(rf);if rfn.starts_with("_bitfield_align"){continue}
                let target=gv.iter().find(|x|clean(&fld(x))==clean(&rfn)&&!clean(&rfn).starts_with('@')).or_else(||{
                    if clean(&rfn)=="@anon"{let ri=rv.iter().take_while(|x|fld(x)!=rfn).filter(|x|clean(&fld(x))=="@anon").count();gv.iter().filter(|x|clean(&fld(x))=="@anon").nth(ri)}else{None}
                });
                if let Some(gf)=target{fm.insert((rn.clone(),rfn),fld(gf));if let (Some(r),Some(g))=(basename(&rf.ty),basename(&gf.ty)){let r=canon(&r,&al);let g=canon(&g,&gal);if rec.contains_key(&r)&&grec.contains_key(&g){map.entry(r).or_insert(g);}}}
            }
        }
        for(rn,rv)in &rec{if map.contains_key(rn){continue}let sig=signature(rv);let mut cand:Vec<_>=grec.iter().filter(|(_,v)|signature(v)==sig).collect();if cand.len()>1{cand.retain(|(_,v)|rv.len()==v.len()&&rv.iter().zip(v.iter()).all(|(r,g)|primsig(&r.ty,&al)==primsig(&g.ty,&gal)));}cand.sort_by_key(|(n,_)|*n);if !cand.is_empty(){map.insert(rn.clone(),cand[0].0.clone());}}
    }
    let unmapped:BTreeMap<_,_>=rec.iter().filter(|(n,_)|!map.contains_key(*n)).map(|(n,v)|(n.clone(),signature(v))).collect();
    std::fs::write(format!("{}.unmapped.json",args[3]),serde_json::to_string_pretty(&unmapped).unwrap()).unwrap();
    std::fs::write(format!("{}.mapping.json",args[3]),serde_json::to_string_pretty(&map).unwrap()).unwrap();
    eprintln!("mapped {} / {} records; {} field names",map.len(),rec.len(),fm.len());
    let zeros:HashMap<String,Expr>=f.items.iter().filter_map(|i|if let Item::Const(c)=i{Some((c.ident.to_string(),(*c.expr).clone()))}else{None}).collect();
    let genconsts:HashMap<String,Type>=gf.items.iter().filter_map(|i|if let Item::Const(c)=i{Some((c.ident.to_string(),(*c.ty).clone()))}else{None}).collect();
    let mut globals=HashMap::new();let mut returns=HashMap::new();let mut ext=vec![];let mut data=vec![];let mut va=HashMap::new();
    for i in &f.items{match i{
        Item::Static(s)=>{globals.insert(s.ident.to_string(),(*s.ty).clone());},
        Item::Fn(s)=>{if let ReturnType::Type(_,t)=&s.sig.output{returns.insert(s.sig.ident.to_string(),(**t).clone());}},
        Item::ForeignMod(m)=>for i in &m.items{match i{
            ForeignItem::Fn(fn_)=>{if let ReturnType::Type(_,t)=&fn_.sig.output{returns.insert(fn_.sig.ident.to_string(),(**t).clone());}if fn_.sig.variadic.is_some(){va.insert(fn_.sig.ident.to_string(),fn_.sig.inputs.len());}ext.push(fn_.clone());},
            ForeignItem::Static(s)=>{globals.insert(s.ident.to_string(),(*s.ty).clone());data.push(s.clone());},_=>{}}},_=>{}}
    }
    map.insert("uintptr_t".into(),"usize".into());map.insert("gc_size_t".into(),"usize".into());
    let mut fix=Fix{records:rec,aliases:al,map,fieldmap:fm,vars:HashMap::new(),globals,returns,va,zeros,fnmap:lift.names,current:String::new(),fused:BTreeMap::new()};
    let mut extout=String::new();let mut plain=String::new();let mut vars=String::new();
    let real=["memcpy","memset","memmove","memcmp","strcmp","strlen","strchr","strcat","strcpy"];
    for mut ex in ext{
        if let Some(n)=fix.fnmap.get(&ex.sig.ident.to_string()){ex.sig.ident=syn::parse_str(n).unwrap();}
        let n=ex.sig.ident.to_string();if real.contains(&n.as_str()){fix.visit_signature_mut(&mut ex.sig);plain.push_str(&format!("{};\n",ts(&ex.sig)));continue}
        for (i,a) in ex.sig.inputs.iter_mut().enumerate(){if let FnArg::Typed(a)=a{let id=Ident::new(&format!("a{i}"),proc_macro2::Span::call_site());a.pat=Box::new(parse_quote!(#id));}}
        fix.visit_signature_mut(&mut ex.sig);
        ex.sig.variadic=None;
        let output=match &ex.sig.output{ReturnType::Default=>"()".to_string(),ReturnType::Type(_,t)=>ts(t)};
        let params=ex.sig.inputs.iter().map(ts).collect::<Vec<_>>().join(", ");
        if output=="!"{extout.push_str(&format!("pub unsafe extern \"C\" fn {n}({params}) -> ! {{ panic!(\"TODO(ext) {n}\") }}\n"));}
        else if fix.va.contains_key(&n){extout.push_str(&format!("crate::ext_va! {{fn {n}({params}) va 0 -> {output};}}\n"));}
        else{extout.push_str(&format!("crate::oext_fns! {{fn {n}({params}) -> {output};}}\n"));}
    }
    let mut globalmeta=vec![];
    for d in data{let n=d.ident;let mut t=*d.ty;fix.visit_type_mut(&mut t);vars.push_str(&format!("pub static mut {n}: {} = unsafe {{ core::mem::zeroed() }};\n",ts(&t)));globalmeta.push(serde_json::json!({"name":n.to_string(),"type":ts(&t)}));}
    f.items.retain(|i|!matches!(i,Item::Struct(_)|Item::Union(_)|Item::Type(_)|Item::ForeignMod(_)|Item::Use(_))&&!matches!(i,Item::Const(c) if genconsts.contains_key(&c.ident.to_string())));
    for i in &mut f.items{if let Item::Static(s)=i{s.vis=parse_quote!(pub);s.attrs.retain(|a|!a.path().is_ident("no_mangle"));}}
    fix.visit_file_mut(&mut f);
    std::fs::write(format!("{}.fused.json",args[3]),serde_json::to_string_pretty(&fix.fused).unwrap()).unwrap();
    let mut fnmeta=vec![];
    for i in &f.items{match i{
        Item::Static(s) if s.ident!="INIT_ARRAY"=>globalmeta.push(serde_json::json!({"name":s.ident.to_string(),"type":ts(&s.ty)})),
        Item::Fn(s) if s.sig.ident!="run_static_initializers"=>{let a=s.sig.inputs.iter().filter_map(|a|if let FnArg::Typed(a)=a{Some(serde_json::json!({"name":ts(&a.pat),"type":ts(&a.ty)}))}else{None}).collect::<Vec<_>>();let r=match &s.sig.output{ReturnType::Default=>"()".to_string(),ReturnType::Type(_,t)=>ts(t)};fnmeta.push(serde_json::json!({"name":s.sig.ident.to_string(),"args":a,"return":r}));},_=>{}
    }}
    std::fs::write(format!("{}.meta.json",args[3]),serde_json::to_string_pretty(&serde_json::json!({"functions":fnmeta,"globals":globalmeta})).unwrap()).unwrap();
    let hdr="//! Native little-endian port of pc/gameworld/script_game.c and its included modules.\n#![allow(non_snake_case,non_camel_case_types,non_upper_case_globals,unused_mut,unused_variables,unused_assignments,unused_unsafe,unused_parens,dead_code,static_mut_refs,clippy::all)]\nuse melee_types::*;\n";
    let out=format!("{hdr}\npub mod ext {{ use super::*; {extout} }}\nuse ext::*;\nextern \"C\" {{ {plain} }}\n{vars}\n{}",prettyplease::unparse(&f));
    let out=out.replace("-10 as FighterKind","(-10i32) as FighterKind");
    std::fs::write(&args[3],out).unwrap();
}
