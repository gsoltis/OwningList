OwningList
----------

OwningList is a linked list that owns its data
but exposes pointers to its nodes. This allows for 
an external data structure to provide "indexing" of the 
list's data. The intended use case is to accumulate 
indexed data, possibly rearranging or removing, and
then finally consuming via `.into_iter()`.

Questions
---------
 1. Why not index over borrows of the data?
    
    I wanted to be able to `Send` the data, which can
    be accomplished safely at consumption time by dropping 
    the index so that there are no pointers outstanding.
 2. Isn't there a better way of accomplishing this? Maybe with Arenas?
    
    Perhaps! I'm definitely open to suggestions.
