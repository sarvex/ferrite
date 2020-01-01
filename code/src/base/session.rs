
use std::pin::Pin;
use std::future::Future;
use async_std::sync::Sender;

use crate::base::process::{ Process };
use crate::base::processes::{ Processes };

/// A session builder is a consumer for the given list of
/// input processes and output a process with given Out type.
pub type Session < P > = 
  PartialSession < (), P >;

pub struct PartialSession
  < I, P >
where
  P: Process,
  I: Processes
{
  executor : Box <
    dyn FnOnce( I::Values, Sender < P::Value > )
      -> Pin < Box < dyn Future < Output=() > + Send > >
    + Send
  >
}

pub fn create_partial_session 
  < I, P, Func, Fut >
  (
    executor : Func
  ) ->
    PartialSession < I, P >
where
  P: Process + 'static,
  I: Processes + 'static,
  Func : 
    FnOnce( I::Values, Sender < P::Value > )
      -> Fut
    + Send + 'static,
  Fut :
    Future < Output=() > + Send
{
  let executor2
    : Box <
        dyn FnOnce( I::Values, Sender < P::Value > )
          -> Pin < Box < dyn Future < Output=() > + Send > >
        + Send
      >
    = Box::new (
        move | ins, sender | {
          Box::pin ( async {
            executor ( ins, sender ).await;
          } )
        });

  PartialSession {
    executor : executor2
  }
}

pub async fn run_partial_session
  < I, P >
  ( session : PartialSession < I, P >
  , ins : I :: Values
  , sender : Sender < P :: Value >
  )
where
  P: Process,
  I: Processes
{
  (session.executor)(ins, sender).await;
}