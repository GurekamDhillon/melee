/* Included after Geno's jdoc reader. Malformed controls atomically fall back to
 * the universal Both/Default/no-extras behavior; they never shift row indexes. */
static int gn_controls_button(const char* name)
{
    int i;
    for(i=0;i<GW_FC_BUTTONS;++i) if(!strcmp(name,gw_fc_labels[i])) return i;
    return -1;
}
static int gn_controls_dest(const jdoc* d, int node, unsigned short* out)
{
    int source, c, n=0;
    unsigned value=0;
    if(d->n[node].type==JN_STR) {
        if(!strcmp(d->n[node].str,"none")) { *out=0; return 1; }
        source=gn_controls_button(d->n[node].str);
        if(source<0) return 0;
        *out=gw_fc_bits[source]; return 1;
    }
    if(d->n[node].type!=JN_ARR || !d->n[node].count || d->n[node].count>GW_FC_BUTTONS) return 0;
    for(c=d->n[node].first;c>=0;c=d->n[c].next) {
        if(d->n[c].type!=JN_STR || (source=gn_controls_button(d->n[c].str))<0 ||
            (value&gw_fc_bits[source])) return 0;
        value|=gw_fc_bits[source]; ++n;
    }
    *out=(unsigned short)value; return n>0;
}
static int gn_controls_row(GwFighterControlsMap* row, const jdoc* d, int node)
{
    int name, buttons, c, source;
    size_t n;
    if(d->n[node].type!=JN_OBJ || d->n[node].count!=2) return 0;
    name=jd_get(d,node,"name"); buttons=jd_get(d,node,"buttons");
    if(name<0 || d->n[name].type!=JN_STR || buttons<0 || d->n[buttons].type!=JN_OBJ ||
       !d->n[buttons].count || d->n[buttons].count>GW_FC_BUTTONS) return 0;
    n=strlen(d->n[name].str);
    if(!n || n>=sizeof row->name || d->n[name].str[0]==' ' || d->n[name].str[n-1]==' ') return 0;
    for(c=0;c<(int)n;++c) if((unsigned char)d->n[name].str[c]<32 || d->n[name].str[c]==127) return 0;
    memset(row,0,sizeof *row); strcpy(row->name,d->n[name].str);
    for(c=d->n[buttons].first;c>=0;c=d->n[c].next) {
        source=gn_controls_button(d->n[c].key);
        if(source<0 || (row->specified&(1u<<source)) || !gn_controls_dest(d,c,&row->dest[source])) return 0;
        row->specified|=1u<<source;
    }
    return 1;
}
static int gn_controls_parse(GwFighterControlsSchema* out, const jdoc* d, int node)
{
    GwFighterControlsSchema result;
    int field, c, i;
    unsigned seen=0;
    memset(out,0,sizeof *out); memset(&result,0,sizeof result);
    if(node<0 || d->n[node].type!=JN_OBJ) return 0;
    for(field=d->n[node].first;field>=0;field=d->n[field].next) {
        int layouts=!strcmp(d->n[field].key,"layouts"), extras=!strcmp(d->n[field].key,"extras");
        int limit=layouts ? GW_FC_MAX_LAYOUTS : GW_FC_MAX_EXTRAS;
        int* count=layouts ? &result.layouts : &result.extras;
        GwFighterControlsMap* rows=layouts ? result.layout : result.extra;
        unsigned bit=layouts ? 1u : 2u;
        if((!layouts && !extras) || d->n[field].type!=JN_ARR || d->n[field].count>limit || (seen&bit)) return 0;
        seen|=bit;
        for(c=d->n[field].first;c>=0;c=d->n[c].next) {
            if(!gn_controls_row(&rows[*count],d,c)) return 0;
            for(i=0;i<*count;++i) if(!strcmp(rows[i].name,rows[*count].name)) return 0;
            ++*count;
        }
    }
    *out=result; return 1;
}
