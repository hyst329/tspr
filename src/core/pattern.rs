use std::collections::VecDeque;

type Idx = u64;

#[derive(Clone)]
struct IdxValue<T>
where
    T: Clone,
{
    index: Idx,
    start: Idx,
    end: Idx,
    value: Result<T>,
}

impl<T: Clone> IdxValue<T> {
    fn new_simple(index: Idx, value: Result<T>) -> IdxValue<T> {
        IdxValue {
            index,
            start: index,
            end: index,
            value,
        }
    }
    fn new(index: Idx, start: Idx, end: Idx, value: Result<T>) -> IdxValue<T> {
        IdxValue {
            index,
            start,
            end,
            value,
        }
    }
}

type QI<T> = VecDeque<IdxValue<T>>;

trait IdxExtractor<Event> {
    fn extract(&self, event: &Event) -> Idx;
}

trait FieldExtractor<Event, T> {
    fn extract(&self, event: &Event, key: &str) -> T;
}

#[derive(Clone)]
enum Result<T> {
    Success(T),
    Failure,
}

trait Pattern<Event, State, T>
where
    State: PatternState<T>,
    T: Clone,
{
    fn initial_state() -> State;
    fn apply(&self, old_state: State, events: Vec<Event>) -> State;
}

trait PatternState<T>
where
    T: Clone,
{
    fn queue(&self) -> QI<T>;
    fn copy_with_queue(&self, queue: QI<T>) -> Self;
}

struct SimplePattern<Event, T> {
    f: Box<Fn(&Event) -> Result<T>>,
    extractor: Box<IdxExtractor<Event>>,
}

struct SimplePatternState<T>
where
    T: Clone,
{
    queue: QI<T>,
}

impl<T: Clone> PatternState<T> for SimplePatternState<T> {
    fn queue(&self) -> QI<T> {
        self.queue.clone()
    }
    fn copy_with_queue(&self, queue: QI<T>) -> SimplePatternState<T> {
        SimplePatternState { queue }
    }
}

impl<Event, T: Clone> Pattern<Event, SimplePatternState<T>, T> for SimplePattern<Event, T> {
    fn initial_state() -> SimplePatternState<T> {
        SimplePatternState {
            queue: VecDeque::new(),
        }
    }

    fn apply(&self, old_state: SimplePatternState<T>, events: Vec<Event>) -> SimplePatternState<T> {
        let queue = events
            .iter()
            .map(|e| IdxValue::new_simple(self.extractor.extract(e), (self.f)(e)))
            .fold(old_state.queue().clone(), |mut v, b| {
                v.push_back(b);
                v
            });
        SimplePatternState { queue }
    }
}

fn constant<Event, T: 'static + Clone>(
    value: T,
    extractor: Box<IdxExtractor<Event>>,
) -> SimplePattern<Event, T> {
    SimplePattern {
        f: Box::new(move |_event| Result::Success(value.clone())),
        extractor,
    }
}

fn field<Event: 'static, T: 'static + Clone>(
    key: String,
    extractor: Box<IdxExtractor<Event>>,
    field_extractor: Box<FieldExtractor<Event, T>>,
) -> SimplePattern<Event, T> {
    SimplePattern {
        f: Box::new(move |event| Result::Success(field_extractor.extract(event, &key))),
        extractor,
    }
}
