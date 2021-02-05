// extern crate log;

use tokio::task;
use async_macros::join;
use std::future::Future;
use std::marker::PhantomData;

use super::lock::Lock;
use super::fix::SharedRecApp;
use super::protocol::SharedProtocol;
use super::linear_to_shared::LinearToShared;
use super::shared_to_linear::SharedToLinear;
use super::shared_session::*;

use crate::base::*;
use crate::protocol::{End};
use crate::functional::nat::*;

pub fn run_shared_session < A >
  ( session : SharedSession < A > )
  ->
  ( SharedChannel < A >
  , impl Future < Output = () >
  )
where
  A : SharedProtocol
{
  let (sender1, receiver1) = unbounded();

  let ( session2, receiver2 ) = unsafe_create_shared_channel ();

  spawn(async move {
    // debug!("[run_shared_session] exec_shared_session");
    unsafe_run_shared_session ( session, receiver1 ).await;
    // debug!("[run_shared_session] exec_shared_session returned");
  });

  let fut = spawn(async move {
    loop {
      let m_senders = receiver2.recv().await;

      debug!("[run_shared_session] received sender3");
      match m_senders {
        Some ( senders ) => {
          sender1.send(senders).unwrap();
        },
        None => {
          debug!("[run_shared_session] terminating shared session");
          return;
        }
      }
    }
  });

  (session2, async { fut.await.unwrap(); })
}

pub fn accept_shared_session
  < F >
  ( cont : PartialSession <
      (Lock < F >, ()),
      F :: Applied
    >
  ) ->
    SharedSession <
      LinearToShared < F >
    >
where
  F : Protocol,
  F : SharedRecApp < SharedToLinear < F > >,
  F::Applied : Protocol
{
  unsafe_create_shared_session (
    move |
      receiver1 :
        Receiver < SharedPayload < LinearToShared < F > > >
    | async move {
      let (sender2, receiver2)
        : (SenderOnce < Lock < F > >, _)
        = once_channel();

      let (sender4, receiver4)
        = once_channel::< F :: Applied >();

      let m_sender1 = receiver1.recv().await;

      match m_sender1 {
        Some (( sender5, sender6, sender7 )) => {
          let child1 = spawn ( async move {
            debug!("[accept_shared_session] calling cont");
            unsafe_run_session
              ( cont, (receiver2, ()), sender4 ).await;
            debug!("[accept_shared_session] returned from cont");
          });

          let child2 = spawn ( async move {
            let linear = receiver4.recv().await.unwrap();
            debug!("[accept_shared_session] received from receiver4");
            sender6.send ( LinearToShared { linear: linear } ).unwrap();
          });

          let child3 = spawn ( async move {
            sender5.send( () ).unwrap();
          });

          let child4 = spawn ( async move {
            debug!("[accept_shared_session] sending sender12");
            sender2.send( Lock {
              unlock : receiver1,
              release: sender7,
            } ).unwrap();
            debug!("[accept_shared_session] sent sender12");
          });

          let _ = join! ( child1, child2, child3, child4 ).await;
        },
        None => {
          // shared session is terminated with all references to it
          // being dropped
        }
      }
    })
}

pub fn detach_shared_session
  < F, C >
  ( cont : SharedSession <
      LinearToShared < F >
    >
  ) ->
    PartialSession <
      (Lock < F >, C),
      SharedToLinear < F >
    >
where
  F : Protocol,
  F : SharedRecApp < SharedToLinear < F > >,
  F::Applied : Protocol,
  C : EmptyContext
{
  unsafe_create_session (
    move |
      (receiver1, _) :
        ( ReceiverOnce <
            Lock < F >
          >,
          C :: Endpoints
        ),
      sender1
    | async move {
      let (sender3, receiver3) = once_channel::<()>();

      let child1 = spawn ( async move {
        debug!("[detach_shared_session] receiving sender2");

        let Lock { unlock : receiver2, release }
          = receiver1.recv().await.unwrap();

        receiver3.recv().await.unwrap();
        release.send(()).unwrap();

        debug!("[detach_shared_session] received sender2");
        unsafe_run_shared_session ( cont, receiver2 ).await;
        debug!("[detach_shared_session] ran cont");
      });

      let child2 = spawn ( async move {
        debug!("[detach_shared_session] sending sender1");
        sender1.send (
          SharedToLinear {
            unlock: sender3,
            phantom: PhantomData
          }
        ).unwrap();
        debug!("[detach_shared_session] sent sender1");
      });

      let _ = join! ( child1, child2 ).await;
    })
}

pub fn async_acquire_shared_session
  < F, Fut >
  ( shared : SharedChannel <
      LinearToShared < F >
    >,
    cont_builder : impl
      FnOnce ( Z ) -> Fut
      + Send + 'static
  ) ->
    task::JoinHandle<()>
where
  F : Protocol,
  F : SharedRecApp < SharedToLinear < F > >,
  Fut :
    Future <
      Output =
        PartialSession < (
          F::Applied , () ),
          End
        >
    > + Send,
  F::Applied : Protocol,
{
  let (receiver3, receiver4, receiver5) = unsafe_receive_shared_channel(shared);

  spawn(async move {
    let (sender1, receiver1) = once_channel();
    let (sender2, receiver2) = once_channel();

    let cont = cont_builder ( Z ).await;

    let ctx = (receiver2, ());

    let child1 = spawn ( async move {
      let LinearToShared { linear } = receiver4.recv().await.unwrap();
      sender2.send(linear).unwrap();
    });

    let child2 = spawn ( async move {
      unsafe_run_session
        ( cont, ctx, sender1
        ).await;
    });

    let child3 = spawn( async move {
      receiver1.recv().await.unwrap();
    });

    let child4 = spawn( async move {
      receiver3.recv().await.unwrap();
      receiver5.recv().await.unwrap();
    });

    let _ = join! (child1, child2, child3, child4).await;
  })
}

pub fn acquire_shared_session
  < F, C, A, Fut >
  ( shared : SharedChannel <
      LinearToShared < F >
    >,
    cont_builder : impl
      FnOnce
        ( C :: Length )
        -> Fut
      + Send + 'static
  ) ->
    PartialSession < C, A >
where
  C : Context,
  A : Protocol,
  F : Protocol,
  F : SharedRecApp < SharedToLinear < F > >,
  C :
    AppendContext <
      ( F::Applied , () )
    >,
  Fut :
    Future <
      Output =
        PartialSession <
          C :: Appended,
          A
        >
    > + Send,
  F::Applied : Protocol,
{
  unsafe_create_session (
    move | ctx1, sender1 | async move {
      let cont = cont_builder (
        < C::Length as Nat > :: nat ()
      ).await;

      let (sender2, receiver2) = once_channel();

      debug!("[acquire_shared_session] receiving receiver3");
      let (receiver3, receiver4, receiver5) = unsafe_receive_shared_channel(shared);
      debug!("[acquire_shared_session] received receiver3");

      receiver3.recv().await.unwrap();

      let ctx2 = C :: append_context
        ( ctx1, (receiver2, ()) );

      let child1 = spawn ( async move {
        let LinearToShared { linear } = receiver4.recv().await.unwrap();
        sender2.send(linear).unwrap();
      });

      let child2 = spawn ( async move {
        unsafe_run_session
          ( cont, ctx2, sender1
          ).await;

        receiver5.recv().await.unwrap();
      });

      let _ = join! (child1, child2) .await;

      // debug!("[acquire_shared_session] ran cont");
    })
}

pub fn release_shared_session
  < F, C, A, N >
  ( _ : N,
    cont :
      PartialSession <
        N :: Target,
        A
      >
  ) ->
    PartialSession < C, A >
where
  A : Protocol,
  C : Context,
  F : Protocol,
  F : SharedRecApp < SharedToLinear < F > >,
  N :
    ContextLens <
      C,
      SharedToLinear < F >,
      Empty
    >,
{
  unsafe_create_session (
    move | ctx1, sender1 | async move {
      let (receiver2, ctx2) = N :: extract_source ( ctx1 );

      let ctx3 = N :: insert_target ( (), ctx2 );

      debug!("[release_shared_session] waiting receiver2");

      let lock: SharedToLinear<F> =
        receiver2.recv().await.unwrap();

      lock.unlock.send(()).unwrap();

      debug!("[release_shared_session] received receiver2");
      unsafe_run_session
        ( cont, ctx3, sender1
        ).await;

      debug!("[release_shared_session] ran cont");
    })
}

impl < F >
  SharedChannel <
    LinearToShared < F >
  >
where
  F : Protocol,
  F : SharedRecApp < SharedToLinear < F > >,
  F::Applied : Protocol,
{
  pub fn acquire < C, A, Fut >
    ( &self,
      cont : impl
        FnOnce
          ( C :: Length )
          -> Fut
        + Send + 'static
    ) ->
      PartialSession < C, A >
  where
    C : Context,
    A : Protocol,
    C :
      AppendContext <
        ( F::Applied , () )
      >,
    Fut :
      Future <
        Output =
          PartialSession <
            C :: Appended,
            A
          >
      > + Send,
  { acquire_shared_session (
      self.clone(),
      cont )
  }
}
