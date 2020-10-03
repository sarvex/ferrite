use async_std::sync::{ channel };

use crate::base::{
  Protocol,
  Context,
  ContextLens,
  wrap_applied,
  PartialSession,
  unsafe_run_session,
  unsafe_create_session,
};

use crate::protocol::choice::nary::*;

pub fn choose
  < N, M, C, A, B, Row >
  ( _ : N,
    _ : M,
    cont : PartialSession < N::Target, A >
  ) ->
    PartialSession < C, A >
where
  C : Context,
  A : Protocol,
  B : Protocol,
  Row : Send + 'static,
  Row : RowCon,
  N :
    ContextLens <
      C,
      ExternalChoice < Row >,
      B
    >,
  M :
    Prism <
      Row,
      Elem = B
    >,
{
  unsafe_create_session (
    async move | ctx1, sender1 | {
      let (receiver1, ctx2) = N::extract_source(ctx1);

      let choice : AppliedSum < Row, () > =
        M::inject_elem ( wrap_applied( () ) );

      let ExternalChoice { sender: sender2 } =
        receiver1.recv().await.unwrap();

      let (sender3, receiver3) = channel(1);

      sender2.send((choice, sender3)).await;

      let receiver_sum = receiver3.recv().await.unwrap();

      let m_receiver =
        M::extract_elem(receiver_sum);

      match m_receiver {
        Some(receiver4) => {
          let ctx3 = N::insert_target(
            *receiver4.get_applied(),
            ctx2
          );
          unsafe_run_session ( cont, ctx3, sender1 ).await;
        },
        None => {
          panic!(
            "impossible happened: received mismatch choice from provider");
        }
      }
    })
}