# An Async channel for those who like abusing MPSC channels

This channel works similarly to tokio's MSPC channel with one trick up it's sleeve.

Instead of reading message one at a time, it can clear the entire channel in a single operation by flipping the pointer between two buffer, making it really efficient for situations where you have a lot of producer and a single consumer.

this library was inspired by this article: https://blog.digital-horror.com/blog/how-to-avoid-over-reliance-on-mpsc/


## Benchmarks
