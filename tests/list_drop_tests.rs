use lisp_bytecode_vm::{List, Value};
use std::sync::Arc;

/// Test dropping small lists (baseline - should always work)
#[test]
fn test_drop_small_list() {
    let list = List::from_vec(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]);
    drop(list); // Should not panic
}

/// Test dropping an empty list
#[test]
fn test_drop_empty_list() {
    let list = List::nil();
    drop(list); // Should not panic
}

/// Test dropping a medium list (100 items)
#[test]
fn test_drop_medium_list() {
    let items: Vec<Value> = (0..100).map(Value::Integer).collect();
    let list = List::from_vec(items);
    drop(list); // Should not panic
}

/// Test dropping a large list (100k items) - this would overflow with recursive drop
#[test]
fn test_drop_large_list() {
    let items: Vec<Value> = (0..100_000).map(|n| Value::Integer(n as i64)).collect();
    let list = List::from_vec(items);
    drop(list); // Should not overflow stack
}

/// Test dropping a very large list (500k items)
#[test]
fn test_drop_very_large_list() {
    let items: Vec<Value> = (0..500_000).map(|n| Value::Integer(n as i64)).collect();
    let list = List::from_vec(items);
    drop(list); // Should not overflow stack
}

/// Test dropping a list with 1 million items
#[test]
fn test_drop_million_item_list() {
    let items: Vec<Value> = (0..1_000_000).map(|n| Value::Integer(n as i64)).collect();
    let list = List::from_vec(items);
    drop(list); // Should not overflow stack
}

/// Test dropping a list when there's a shared tail (structural sharing)
/// When l1 is dropped, the shared tail should NOT be freed because l2 still references it
#[test]
fn test_drop_shared_tail() {
    // Create a shared tail
    let tail_items: Vec<Value> = (0..10_000).map(|n| Value::Integer(n as i64)).collect();
    let tail = List::from_vec(tail_items);

    // Create two lists that share this tail
    let l1 = List::cons(Value::Integer(-1), tail.clone());
    let l2 = List::cons(Value::Integer(-2), tail.clone());

    // Drop l1 - should NOT affect l2's access to the shared tail
    drop(l1);

    // l2 and its tail should still be valid
    assert!(!l2.is_nil());

    // The tail should still be accessible
    let l2_tail = l2.cdr().unwrap();
    assert_eq!(l2_tail.len(), 10_000);

    // Drop l2 - now the shared tail can be freed
    drop(l2);

    // The original tail reference is still valid
    assert_eq!(tail.len(), 10_000);
}

/// Test dropping lists with large shared tails
#[test]
fn test_drop_large_shared_tail() {
    // Create a large shared tail (100k items)
    let tail_items: Vec<Value> = (0..100_000).map(|n| Value::Integer(n as i64)).collect();
    let shared_tail = List::from_vec(tail_items);

    // Create 10 lists all sharing the same tail
    let lists: Vec<List> = (0..10)
        .map(|i| List::cons(Value::Integer(i), shared_tail.clone()))
        .collect();

    // Drop all but one list
    for list in lists.into_iter().take(9) {
        drop(list);
    }

    // The shared tail should still exist (referenced by original shared_tail)
    assert_eq!(shared_tail.len(), 100_000);
}

/// Test that dropping a list with nested lists works correctly
/// This tests the "nested Value" concern - if head contains another list,
/// it will use recursive drop for the inner list. This is acceptable for
/// moderately nested lists.
#[test]
fn test_drop_nested_lists() {
    // Create a list of lists (moderately nested)
    let inner_lists: Vec<Value> = (0..100)
        .map(|_| {
            let inner = List::from_vec(vec![Value::Integer(1), Value::Integer(2)]);
            Value::List(inner)
        })
        .collect();

    let outer = List::from_vec(inner_lists);
    drop(outer); // Should not panic
}

/// Test that creating and dropping many lists doesn't leak memory
/// (This is more of a sanity check than a precise memory test)
#[test]
fn test_drop_many_lists_no_leak() {
    for _ in 0..100 {
        let items: Vec<Value> = (0..10_000).map(|n| Value::Integer(n as i64)).collect();
        let list = List::from_vec(items);
        drop(list);
    }
    // If we get here without OOM, we're probably not leaking badly
}

/// Test Arc strong_count behavior with shared tails
#[test]
fn test_arc_refcount_behavior() {
    let tail = List::from_vec(vec![Value::Integer(1), Value::Integer(2)]);

    // Get Arc from tail
    let arc1 = match &tail {
        List::Cons(arc) => Arc::strong_count(arc),
        List::Nil => panic!("Expected Cons"),
    };
    assert_eq!(arc1, 1);

    // Clone the tail - Arc count should increase
    let tail2 = tail.clone();
    let arc2 = match &tail {
        List::Cons(arc) => Arc::strong_count(arc),
        List::Nil => panic!("Expected Cons"),
    };
    assert_eq!(arc2, 2);

    // Drop one clone
    drop(tail2);
    let arc3 = match &tail {
        List::Cons(arc) => Arc::strong_count(arc),
        List::Nil => panic!("Expected Cons"),
    };
    assert_eq!(arc3, 1);
}
