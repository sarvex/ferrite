use crate::protocol::choice::nary::*;
use crate::protocol::choice::nary::either::*;
use crate::session::choice::nary;
use crate::session::choice::nary::internal_session::*;
use crate::session::choice::nary::internal_choice_case as choice;

use crate::protocol::choice::binary::{
  InternalChoice
};

use crate::base::{
  PartialSession,
  Protocol,
  Context,
  Empty,
  ContextLens,
};

/*
  Additive Disjuction / Internal Choice

  Right Rule (Session)

            cont :: Δ ⊢ P
  =================================
    offer_left(cont) :: Δ ⊢ P ⊕ Q

  offerLeft
    :: forall ctx p q
       ( Protocol p
       , Protocol q
       , Context ctx
       )
    =>  PartialSession ctx p
    ->  PartialSession ctx (InternalChoice p q)
 */
pub fn offer_left
  < C, A, B >
  ( cont:  PartialSession < C, A >
  ) ->
    PartialSession < C,
      InternalChoice < A, B >
    >
where
  A : Protocol,
  B : Protocol,
  C : Context
{
  nary::offer_case ( LeftLabel, cont )
}

pub fn offer_right
  < C, A, B >
  ( cont:  PartialSession < C, B > )
  -> PartialSession < C, InternalChoice < A, B > >
where
  A : Protocol,
  B : Protocol,
  C : Context,
{
  nary::offer_case ( RightLabel, cont )
}

/*
  Additive Disjuction / Internal Choice

  Left Rule (Client)

      cont_builder(Left)  :: Δ, P, Δ' ⊢ S
      cont_builder(Right) :: Δ, Q, Δ' ⊢ S
  ===========================================
    case(cont_builder) :: Δ, P ⊕ Q, Δ' ⊢ S
 */

pub type ContSum < N, C, A1, A2, B, Del > =
  AppliedSum <
    Either < A1, A2 >,
    InternalSessionF <
      N, C, B,
      Either < A1, A2 >,
      Del
    >
  >
;

pub type InjectCont < N, C, A1, A2, B, Del > =
  < Either < A1, A2 >
    as WrapRow <
      choice::InjectSessionApp <
        N, C, B, Either < A1, A2 >, Del
      >
    >
  > :: Unwrapped
;

pub fn case
  < N, C, D, A1, A2, B >
  ( n : N,
    cont : impl
      FnOnce (
        InjectCont < N, C, A1, A2, B, D >
      ) ->
        ContSum < N, C, A1, A2, B, D >
      + Send + 'static
  ) ->
    PartialSession < C, B >
where
  C : Context,
  D : Context,
  A1 : Protocol,
  A2 : Protocol,
  B : Protocol,
  N :
    ContextLens <
      C,
      InternalChoice < A1, A2 >,
      A1,
      Deleted = D
    >,
  N :
    ContextLens <
      C,
      InternalChoice < A1, A2 >,
      A2,
      Deleted = D
    >,
  N :
    ContextLens <
      C,
      InternalChoice < A1, A2 >,
      Empty,
      Deleted = D
    >
{
  nary::case ( n, cont )
}
