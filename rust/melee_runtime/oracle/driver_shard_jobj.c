#include <stdio.h>
void shard_jobj_run(unsigned);
static FILE *out;
void shard_emit(unsigned seed,unsigned stage,unsigned hash){fprintf(out,"%x %x %08x\n",seed,stage,hash);}
int main(int argc,char**argv){unsigned i;out=fopen(argv[1],"w");for(i=0;i<128;i++)shard_jobj_run(i);fclose(out);return 0;}
