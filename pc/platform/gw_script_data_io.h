/* Host data-file reads: errors are values so optional first-result callers work. */
#ifndef GW_SCRIPT_DATA_IO_H
#define GW_SCRIPT_DATA_IO_H
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <string.h>
#include <sys/stat.h>
static int gs_data_exists_path(const char* path, const char** error)
{
    struct stat st;
    *error=NULL;
    if (stat(path,&st)==0) return 1;
    if (errno==ENOENT || errno==ENOTDIR) return 0;
    *error=strerror(errno); return -1;
}
static int gs_data_read_path(const char* path, char** out, size_t* length, const char** error)
{
    FILE* f;
    long n;
    size_t got;
    int saved;
    *out=NULL; *length=0; *error=NULL;
    f=fopen(path,"rb");
    if (!f) { *error=(errno==ENOENT || errno==ENOTDIR)?"missing":strerror(errno); return 0; }
    if (fseek(f,0,SEEK_END)!=0 || (n=ftell(f))<0 || fseek(f,0,SEEK_SET)!=0) {
        saved=errno; fclose(f); *error=saved?strerror(saved):"cannot seek data file"; return 0;
    }
    if (n>(1<<20)) { fclose(f); *error="data file too large"; return 0; }
    *out=(char*)malloc((size_t)n+1);
    if (!*out) { fclose(f); *error="out of memory"; return 0; }
    got=fread(*out,1,(size_t)n,f);
    if (got!=(size_t)n || ferror(f)) {
        saved=errno; fclose(f); free(*out); *out=NULL;
        *error=saved?strerror(saved):"cannot read complete data file"; return 0;
    }
    if (fclose(f)!=0) { saved=errno; free(*out); *out=NULL; *error=strerror(saved); return 0; }
    *length=got; return 1;
}
#endif
