use crate::core::time::Window;
use crate::core::time::Interval;
use crate::core::time::TimeInterval;

enum AggregateFunction {
    Sum,
    Count,
    Avg,
    Lag,
}

enum AST {
    ConstantInt(i64),
    ConstantReal(f64),
    ConstantString(String),
    ConstantBoolean(bool),
    Identifier(String),
    FunctionCall(String, Vec<AST>),
    Range(i64, i64),
    ReducerFunctionCall(String, Vec<AST>),
    AndThen(Box<AST>, Box<AST>),
    Timer(Box<AST>, TimeInterval),
    Assert(Box<AST>),
    ForWithInterval(Box<AST>, Option<bool>, Window, Interval),
    AggregateFunctionCall(AggregateFunction, Box<AST>, Window)
}
