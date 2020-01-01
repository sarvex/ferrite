use async_std::task;
use async_macros::join;
use async_std::sync::{ Sender, Receiver, channel };
use std::future::{ Future };

use crate::process::{ ReceiveValue };
use crate::base::*;

/*
              cont_builder(x) :: Δ ⊢ P
    ==============================================
      send_input(cont_builder) :: Δ ⊢ T ⊃ P
 */
pub fn receive_value
  < T, Ins, P, Func, Fut >
  ( cont_builder : Func ) ->
     PartialSession < Ins, ReceiveValue < T, P > >
where
  T : Send + 'static,
  P : Process + 'static,
  Ins : Processes + 'static,
  Func : 
    FnOnce(T) -> Fut
    + Send + 'static,
  Fut :
    Future <
      Output = PartialSession < Ins, P >
    > + Send
{
  create_partial_session (
    async move |
      ins : Ins::Values,
      sender1 : Sender < (
        Sender < T >,
        Receiver < P::Value >
      ) >
    | {
      let (sender2, receiver2)
        : (Sender < T >, _)
        = channel(1);

      let (sender3, receiver3) = channel(1);

      let child1 = task::spawn ( async move {
        sender1.send((sender2, receiver3)).await;
      });


      let child2 = task::spawn ( async move {
        let val = receiver2.recv().await.unwrap();

        let cont = cont_builder(val).await;
        run_partial_session
          ( cont, ins, sender3
          ).await;
      });

      join!(child1, child2).await;
    })
}

/*
          cont_builder() :: T ; Q, Δ ⊢ P
    ===========================================
      send_value_to_async(cont_builder) :: T ⊃ Q, Δ ⊢ P
 */
pub fn send_value_to_async
  < Lens, Ins1, Ins2, Ins3, P, Q, T, Func, Fut >
  ( _ : Lens,
    cont_builder : Func
  ) ->
    PartialSession < Ins1, P >
where
  P : Process + 'static,
  Q : Process + 'static,
  Ins1 : Processes + 'static,
  Ins2 : Processes + 'static,
  Ins3 : Processes + 'static,
  T : Send + 'static,
  Func : 
    FnOnce() -> Fut
    + Send + 'static,
  Fut :
    Future <
      Output = ( T,  PartialSession < Ins2, P > )
    > + Send,
  Lens :
    ProcessLens <
      Ins1,
      Ins2,
      Ins3,
      ReceiveValue < T, Q >,
      Q
    >
{
  create_partial_session ( 
    async move |
      ins1: < Ins1 as Processes >::Values,
      sender1 : Sender < P::Value >
    | {
      let (receiver1, ins2) =
        < Lens as
          ProcessLens < Ins1, Ins2, Ins3, ReceiveValue < T, Q >, Q >
        >
        :: split_channels ( ins1 );

      let (sender2, receiver2) = receiver1.recv().await.unwrap();
      let (val, cont) = cont_builder().await;

      let ins3 =
        < Lens as
          ProcessLens < Ins1, Ins2, Ins3, ReceiveValue < T, Q >, Q >
        >
        :: merge_channels( receiver2, ins2);

      let child1 = task::spawn(async move {
        sender2.send(val).await;
      });

      let child2 = task::spawn(async move {
        run_partial_session
          (cont, ins3, sender1
          ).await;
      });

      join!(child1, child2).await;
    })
}

pub fn send_value_to
  < Lens, I1, I2, I3, P, Q, T >
  ( lens : Lens,
    value : T,
    cont : PartialSession < I2, P >
  ) ->
    PartialSession < I1, P >
where
  P : Process + 'static,
  Q : Process + 'static,
  I1 : Processes + 'static,
  I2 : Processes + 'static,
  I3 : Processes + 'static,
  T : Send + 'static,
  Lens :
    ProcessLens <
      I1,
      I2,
      I3,
      ReceiveValue < T, Q >,
      Q
    >
{
  send_value_to_async ( lens, async || {
    ( value, cont )
  })
}
