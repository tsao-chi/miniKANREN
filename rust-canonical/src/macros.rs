//! Macros for embedding miniKANREN as DSL in Rust

/// Creates a goal that succeeds if any of its subgoals succeeds
#[macro_export]
macro_rules! disj {
    () => { fail() };
    ($g:expr) => { $g };
    ($g0:expr; $($g:expr);*) => { disj2($g0, disj!($($g);*))}
}

/// Creates a goal that succeeds if all of its subgoals succeed
#[macro_export]
macro_rules! conj {
    () => { succeed() };
    ($g:expr) => { $g };
    ($g0:expr, $($g:expr),*) => { conj2($g0, conj!($($g),*))}
}

/// Define a relation.
/// A relation is a function that creates a goal.
#[macro_export]
macro_rules! defrel {
    ($(#[$outer:meta])* pub $name:ident($($args:ident),*) { $($g:expr),* $(,)? }) => {
        $(#[$outer])*
        pub fn $name($($args: impl 'static + Into<Value>),*) -> impl Goal<StatSubs> {
            $(
                let $args = $args.into();
            )*
            move |s| {
                $(
                    let $args = $args.clone();
                )*
                Stream::suspension(move || conj!($($g),*).apply(s))
            }
        }
    };

    ($(#[$outer:meta])* $name:ident($($args:ident),*) { $($g:expr),* $(,)? }) => {
        $(#[$outer])*
        fn $name($($args: impl 'static + Into<Value>),*) -> impl Goal<StatSubs> {
            $(
                let $args = $args.into();
            )*
            move |s| {
                $(
                    let $args = $args.clone();
                )*
                Stream::suspension(move || conj!($($g),*).apply(s))
            }
        }
    };

    // alternate syntax: separate goals with ;
    (pub $name:ident($($args:ident),*) { $($g:expr);* $(;)? }) => {
        defrel!{pub $name($($args),*) { $($g),* }}
    };

    // alternate syntax: separate goals with ;
    ($name:ident($($args:ident),*) { $($g:expr);* $(;)? }) => {
        defrel!{$name($($args),*) { $($g),* }}
    };
}

/// Run one or more goals.
///
/// The syntax `run!(n, var(s), goal1, goal2, ...)` produces at most n
/// solutions in Scheme you wold write `(run n var(s) goal1 goal2 ...)`.
/// The syntax `run!(*, var(s), goal1, goal2, ...)` produces all
/// solutions in Scheme you wold write `(run* var(s) goal1 goal2 ...)`.
/// The latter may result in an infinite recursion which eventually
/// crashes with a stack overflow.
///
/// We support an additional syntax `run!(var(s), goal1, goal2, ...)`
/// that returns a (possibly infinite) iterator over all solutions.
#[macro_export]
macro_rules! run {
    (*, ($($x:ident),*), $($body:tt)*) => {
        run!(@ *, ($($x),*), $($body)*)
    };

    (*, $q:ident, $($g:expr),* $(,)?) => {
        run!(@ *, $q, $($g),*)
    };

    ($n:expr, ($($x:ident),*), $($body:tt)*) => {
        run!(@ $n, ($($x),*), $($body)*)
    };

    ($n:tt, $q:ident, $($g:expr),* $(,)?) => {
        run!(@ $n, $q, $($g),*)
    };

    (($($x:ident),*), $($body:tt)*) => {
        run!(@ iter, ($($x),*), $($body)*)
    };

    ($q:ident, $($g:expr),* $(,)?) => {
        run!(@ iter, $q, $($g),*)
    };

    (@ $n:tt, ($($x:ident),*), $($g:expr),* $(,)?) => {
        run!(@ $n, q, {
            fresh!(
                ($($x),*),
                eq(vec![$(Value::var($x.clone())),*], q),
                $($g),*
            )
        })
    };

    (@ *, $q:ident, $($g:expr),* $(,)?) => {{
        let $q = Var::new(stringify!($q));
        let var = Value::var($q.clone());
        conj!($($g),*).run_inf().map(move |s| s.reify(&var))
    }};

    (@ iter, $q:ident, $($g:expr),* $(,)?) => {{
        let $q = Var::new(stringify!($q));
        let var = Value::var($q.clone());
        conj!($($g),*).iter().map(move |s| s.reify(&var))
    }};

    (@ $n:expr, $q:ident, $($g:expr),* $(,)?) => {{
        let $q = Var::new(stringify!($q));
        let var = Value::var($q.clone());
        conj!($($g),*).run($n).map(move |s| s.reify(&var))
    }};
}

/// Bind fresh variables with scope inside the body of `fresh!`.
#[macro_export]
macro_rules! fresh {
    (($($x:ident),*), $($g:expr),* $(,)?) => {{
        $( let $x = Var::new(stringify!($x)); )*
        conj!($($g),*)
    }}
}

/// Creates a goal that succeeds if any of its *lines* succeeds.
/// Every successful *line* contributes one or more values.
///
/// A *line* (separated by `;`) succeeds if all of its
/// goals (separated by `,`) succeed.
#[macro_export]
macro_rules! conde {
    ( $($($g:expr),*;)* ) => {
        disj!($(conj!( $($g),*));*)
    }
}

/// Creates a goal that succeeds if any of its *lines* succeeds.
/// Only the first *line* that succeeds can contribute values.
///
/// A *line* (separated by `;`) succeeds if all of its
/// goals (separated by `,`) succeed.
#[macro_export]
macro_rules! conda {
    ($($g:expr),*) => { conj!($($g),*) };

    ($g0:expr, $($g:expr),*; $($rest:tt)*) => {
        ifte($g0, conj!($($g),*), conda!($($rest)*))
    };
}

/// `Condu!` behaves like `conda!`, except that a successful line
/// succeeds only once.
#[macro_export]
macro_rules! condu {
    ( $($g0:expr, $($g:expr),*);* ) => {
        conda!($(once($gO), $($g),*);*)
    }
}
