;; Test hash-map constructor and operations

;; Create an empty hashmap
(def empty-map (hash-map))
(print (hashmap? empty-map))

;; Create a hashmap with key-value pairs
(def my-map (hash-map "name" "Alice" "age" 30 "city" "NYC"))

;; Test hashmap-get
(print (hashmap-get my-map "name"))

;; Test hashmap-contains-key?
(print (hashmap-contains-key? my-map "name"))
(print (hashmap-contains-key? my-map "missing"))

;; Test hashmap-set (creates new hashmap)
(def updated-map (hashmap-set my-map "age" 31))
(print (hashmap-get updated-map "age"))

;; Original map is unchanged (immutability)
(print (hashmap-get my-map "age"))

;; Test hashmap-keys
(def keys (hashmap-keys my-map))
(print (list-length keys))

;; Test hashmap-values
(def values (hashmap-values my-map))
(print (list-length values))

;; Test with different value types
(def mixed-map (hash-map "num" 42 "bool" true "list" '(1 2 3)))
(print (hashmap-get mixed-map "num"))
(print (hashmap-get mixed-map "bool"))
(print (hashmap-get mixed-map "list"))
