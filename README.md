# DistributedHashTable
* Assignment 1 for CSE 403

###Chronological progress:
* Decided to use AWS. Spent time reading/learning about EC2 and launching/maintaining instances.

* Decided to work with Rust. Currently shakiest on the topic of lifetimes.

* Will be using CSE 303 server/client codebase as inspiration for how to structure my server/client.

* Learned about binaries and local libraries in Rust. For libraries, you can chain together modules.
Not sure what the best/most sensible structure would be. I imagine this would come with experience.

* Started with basic localhost messaging from client to server and back. :D

* Implemented properties file and file loading using serde to deserialize a json file.

* Attempted to implement server structure using closures (lambdas) but couldn't figure
out my way around the borrow checker :(. This would have been nice since it would allow
my different code in different source files to be more modularized/generalized.
Ended up opting for a more standard normal function calls (instead of passing a function
as a parameter to another function to be called later).
May attempt again in the future once I understand Rust more.

* Previous attempts for socket communication involved byte by byte transmission and handling.
Found out the serde library has support for serializing/deserializing for a tcp stream. Will try using this.

*