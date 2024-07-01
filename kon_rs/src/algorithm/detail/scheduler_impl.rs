use crate::algorithm::{pruning_decorators::ITraverseDecorator, ITreeCallback};

pub struct SchedulerImpl<TDecorator: ITraverseDecorator, TCallback: ITreeCallback> {
    decorator: TDecorator,
    callback: TCallback,
}
