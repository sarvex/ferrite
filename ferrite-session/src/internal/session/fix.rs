use tokio::{
  task,
  try_join,
};

use crate::internal::base::*;

pub fn fix_session<R, F, A, C>(
  cont: PartialSession<C, A>
) -> PartialSession<C, RecX<R, F>>
where
  C: Context,
  R: Context,
  F: Protocol,
  A: Protocol,
  F: RecApp<(RecX<R, F>, R), Applied = A>,
{
  unsafe_create_session::<C, RecX<R, F>, _, _>(move |ctx, sender1| async move {
    let (provider_end_a, consumer_end_a) = A::create_endpoints();

    let child1 = task::spawn(async move {
      let val = receiver.recv().await.unwrap();

      sender1.send(fix(val)).unwrap();
    });

    unsafe_run_session(cont, ctx, provider_end_a);

    try_join!(child1, child2).unwrap();
  })
}

pub fn unfix_session<N, C1, C2, A, B, R, F>(
  _n: N,
  cont: PartialSession<C2, B>,
) -> PartialSession<C1, B>
where
  B: Protocol,
  C1: Context,
  C2: Context,
  F: Protocol,
  R: Context,
  F: RecApp<(RecX<R, F>, R), Applied = A>,
  A: Protocol,
  N: ContextLens<C1, RecX<R, F>, A, Target = C2>,
{
  unsafe_create_session(move |ctx1, sender1| async move {
    let (receiver1, ctx2) = N::extract_source(ctx1);

    let (sender2, receiver2) = once_channel();

    let ctx3 = N::insert_target(receiver2, ctx2);

    let child1 = task::spawn(async move {
      let val = receiver1.recv().await.unwrap();

      sender2.send(unfix(val)).unwrap();
    });

    let child2 = task::spawn(unsafe_run_session(cont, ctx3, sender1));

    try_join!(child1, child2).unwrap();
  })
}
