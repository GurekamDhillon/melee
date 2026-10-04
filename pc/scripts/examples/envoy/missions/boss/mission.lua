-- Mission owns traversal/lives/time. Envoy additionally requires P2's native
-- fall count to rise after room entry. An exit reached early does not win.
return {
  start={x=-100,y=12},
  goal={x=150,y=20,w=24,h=80},
  objective={type='reach_goal',time=180,lives=3},
  triggers={{x=-100,y=20,w=30,h=60,action='message',text='Boss: KO the CPU once, then reach the right exit.'}},
}
