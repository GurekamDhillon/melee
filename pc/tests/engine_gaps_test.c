/* Standalone host fixture; never links or runs the game. */
#include "../platform/gw_script_data_io.h"
#include <assert.h>
#include <direct.h>
int main(int argc,char** argv)
{
    char path[1024], *buf; size_t n; const char* err; FILE* f;
    assert(argc==2);
    snprintf(path,sizeof path,"%s/missing",argv[1]);
    assert(gs_data_exists_path(path,&err)==0 && !err);
    assert(!gs_data_read_path(path,&buf,&n,&err) && !strcmp(err,"missing"));
    snprintf(path,sizeof path,"%s/content",argv[1]);
    f=fopen(path,"wb");assert(f);assert(fwrite("a\0b",1,3,f)==3);fclose(f);
    assert(gs_data_exists_path(path,&err)==1);
    assert(gs_data_read_path(path,&buf,&n,&err) && n==3 && !memcmp(buf,"a\0b",3));free(buf);
    f=fopen(path,"wb");assert(f);fclose(f);
    assert(gs_data_read_path(path,&buf,&n,&err) && n==0);free(buf);
    f=fopen(path,"wb");assert(f);assert(fseek(f,(1<<20),SEEK_SET)==0);fputc(0,f);fclose(f);
    assert(!gs_data_read_path(path,&buf,&n,&err) && !strcmp(err,"data file too large"));
    snprintf(path,sizeof path,"%s/unreadable",argv[1]);assert(_mkdir(path)==0);
    assert(gs_data_exists_path(path,&err)==1);
    assert(!gs_data_read_path(path,&buf,&n,&err) && strcmp(err,"missing"));
    puts("engine gap real data-file fixtures PASS");return 0;
}
