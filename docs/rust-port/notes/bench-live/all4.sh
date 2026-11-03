for i in 1 2 3; do
  for sc in hy fd; do
    if [ $sc = hy ]; then SPEC=fox/cpu9,marth/cpu9,falco/cpu9,peach/cpu9,14; SC=s_hyrule.txt; SF=6241; T="Temple 4P"; else SPEC=fox/cpu9,marth/cpu9,32; SC=s_fd.txt; SF=6228; T="FD 2P"; fi
    rm -rf Q_R_${sc}$i Q_S_${sc}$i
    echo "run $i $sc load: $(powershell -NoProfile -Command "(Get-Counter '\Processor(_Total)\% Processor Time').CounterSamples[0].CookedValue") ; $(tasklist | grep -i -E 'rustc|cargo|melee|cl.exe|link.exe' | awk '{print $1}' | sort | uniq -c | tr '\n' ' ')"
    BTITLE="rust-release $T" ./run_r.sh Q_R_${sc}$i $SPEC 3603
    ./run_s2.sh Q_S_${sc}$i $SC $SF
  done
done
for sc in hy fd; do
  if [ $sc = hy ]; then SPEC=fox/cpu9,marth/cpu9,falco/cpu9,peach/cpu9,14; T="Temple 4P stats"; else SPEC=fox/cpu9,marth/cpu9,32; T="FD 2P stats"; fi
  rm -rf Q_G_$sc; BTITLE="rust-release $T" GXS=1 ./run_r.sh Q_G_$sc $SPEC 3603
done
echo ALLDONE
