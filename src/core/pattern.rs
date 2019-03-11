use std::cmp::Ordering;
use std::collections::VecDeque;
use std::marker::PhantomData;

#[derive(Clone, Copy, Eq)]
struct Idx(u64);

const IDX_MODULUS: u64 = 100_000;

impl Ord for Idx {
    fn cmp(&self, other: &Idx) -> Ordering {
        (self.0 / IDX_MODULUS).cmp(&(other.0 / IDX_MODULUS))
    }
}

impl PartialOrd for Idx {
    fn partial_cmp(&self, other: &Idx) -> Option<Ordering> {
        Some((self.0 / IDX_MODULUS).cmp(&(other.0 / IDX_MODULUS)))
    }
}

impl PartialEq for Idx {
    fn eq(&self, other: &Idx) -> bool {
        (self.0 / IDX_MODULUS) == (other.0 / IDX_MODULUS)
    }
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
enum Result<T> {
    Success(T),
    Failure,
}

trait Pattern<Event, State, T>
where
    State: PatternState<T>,
    T: Clone,
{
    fn initial_state(&self) -> State;
    fn apply(&self, old_state: State, events: Vec<Event>) -> State;
}

trait PatternState<T>: Clone
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

#[derive(Clone)]
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
    fn initial_state(&self) -> SimplePatternState<T> {
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

struct CouplePattern<Event, State1, State2, T1, T2, T3>
where
    State1: PatternState<T1>,
    State2: PatternState<T2>,
    T1: Clone,
    T2: Clone,
    T3: Clone,
{
    left: Box<Pattern<Event, State1, T1>>,
    right: Box<Pattern<Event, State2, T2>>,
    func: Box<Fn(&Result<T1>, &Result<T2>) -> Result<T3>>,
}

#[derive(Clone)]
struct CouplePatternState<State1, State2, T1, T2, T3>
where
    State1: PatternState<T1>,
    State2: PatternState<T2>,
    T1: Clone,
    T2: Clone,
    T3: Clone,
{
    left: Box<State1>,
    right: Box<State2>,
    queue: QI<T3>,
    phantom1: PhantomData<T1>,
    phantom2: PhantomData<T2>,
}

fn couple_inner<T1: Clone, T2: Clone, T3: Clone>(
    mut first: QI<T1>,
    mut second: QI<T2>,
    mut total: QI<T3>,
    func: Box<Fn(&Result<T1>, &Result<T2>) -> Result<T3>>,
) -> (QI<T1>, QI<T2>, QI<T3>) {
    let default = (first.clone(), second.clone(), total.clone());
    match (first.get(0), second.get(0)) {
        (Some(iv1), Some(iv2)) => {
            let idx1 = iv1.index;
            let val1 = &iv1.value;
            let idx2 = iv2.index;
            let val2 = &iv2.value;
            if idx1 == idx2 {
                let result = func(val1, val2);
                first.pop_front();
                second.pop_front();
                total.push_back(IdxValue::new_simple(idx1, result));
                couple_inner(first, second, total, func)
            } else if idx1 < idx2 {
                first.pop_front();
                couple_inner(first, second, total, func)
            } else {
                second.pop_front();
                couple_inner(first, second, total, func)
            }
        }
        _ => default,
    }
}

fn couple_process_queues<T1: Clone, T2: Clone, T3: Clone>(
    first: QI<T1>,
    second: QI<T2>,
    total: QI<T3>,
    func: Box<Fn(&Result<T1>, &Result<T2>) -> Result<T3>>,
) -> (QI<T1>, QI<T2>, QI<T3>) {
    couple_inner::<T1, T2, T3>(first, second, total, func)
}

impl<Event, State1, State2, T1: Clone, T2: Clone, T3: Clone>
    Pattern<Event, CouplePatternState<State1, State2, T1, T2, T3>, T3>
    for CouplePattern<Event, State1, State2, T1, T2, T3>
where
    State1: PatternState<T1>,
    State2: PatternState<T2>,
{
    fn initial_state(&self) -> CouplePatternState<State1, State2, T1, T2, T3> {
        CouplePatternState {
            left: Box::new(self.left.initial_state()),
            right: Box::new(self.right.initial_state()),
            queue: VecDeque::new(),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    fn  apply(&self, old_state: CouplePatternState<State1, State2, T1, T2, T3>, events: Vec<Event>) -> CouplePatternState<State1, State2, T1, T2, T3> {
        unimplemented!()
    }
}

impl<State1, State2, T1: Clone, T2: Clone, T3: Clone> PatternState<T3>
    for CouplePatternState<State1, State2, T1, T2, T3>
where
    State1: PatternState<T1>,
    State2: PatternState<T2>,
{
    fn queue(&self) -> QI<T3> { self.queue.clone() }
    fn copy_with_queue(&self, queue: QI<T3>) -> CouplePatternState<State1, State2, T1, T2, T3> {
       CouplePatternState { left: self.left.clone(), right: self.right.clone(), queue, phantom1: self.phantom1, phantom2: self.phantom2 }
    }
}
