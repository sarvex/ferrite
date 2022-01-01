use core::{
  future::Future,
  pin::Pin,
};

use crate::internal::{
  base::*,
  functional::*,
};

pub struct InternalChoice<Row>
where
  Row: ToRow,
{
  pub(crate) field: AppSum<Row::Row, ReceiverF>,
}

impl<Row1, Row2> Protocol for InternalChoice<Row1>
where
  Row1: Send + 'static,
  Row2: Send + 'static,
  Row1: ToRow<Row = Row2>,
{
  type ConsumerEndpoint = ReceiverOnce<AppSum<Row2, ConsumerEndpointF>>;
  type ProviderEndpoint = SenderOnce<AppSum<Row2, ConsumerEndpointF>>;

  fn create_endpoints() -> (Self::ProviderEndpoint, Self::ConsumerEndpoint)
  {
    once_channel()
  }

  fn forward(
    consumer_end: Self::ConsumerEndpoint,
    provider_end: Self::ProviderEndpoint,
  ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
  {
    Box::pin(async {
      let endpoint = consumer_end.recv().await.unwrap();
      provider_end.send(endpoint).unwrap();
    })
  }
}

impl<Row1, Row2, Row3, A> RecApp<A> for InternalChoice<Row1>
where
  A: Send + 'static,
  Row1: Send + 'static,
  Row1: ToRow<Row = Row2>,
  Row2: RowCon,
  Row2: RecApp<A, Applied = Row3>,
  Row3: RowCon,
{
  type Applied = InternalChoice<RecRow<A, Row1>>;
}

impl<Row1, Row2, Row3, A> SharedRecApp<A> for InternalChoice<Row1>
where
  A: Send + 'static,
  Row1: ToRow<Row = Row2>,
  Row2: RowCon,
  Row2: SharedRecApp<A, Applied = Row3>,
  Row3: RowCon,
{
  type Applied = InternalChoice<SharedRecRow<A, Row1>>;
}

impl<Row1, Row2> ForwardChannel for InternalChoice<Row1>
where
  Row1: Send + 'static,
  Row1: ToRow<Row = Row2>,
  Row2: RowCon,
  AppSum<Row2, ReceiverF>: ForwardChannel,
{
  fn forward_to(
    self,
    sender: OpaqueSender,
    receiver: OpaqueReceiver,
  )
  {
    self.field.forward_to(sender, receiver)
  }

  fn forward_from(
    sender: OpaqueSender,
    receiver: OpaqueReceiver,
  ) -> Self
  {
    InternalChoice {
      field: <AppSum<Row2, ReceiverF>>::forward_from(sender, receiver),
    }
  }
}
