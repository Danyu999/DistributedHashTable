# DistributedHashTable
* Assignment 1 for CSE 403

### Chronological progress:
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
Found out the serde library has support for serializing/deserializing for a tcp stream. Will use this.

* Created enum types for RequestMessages and BarrierMessages to serialize/deserialize to. Starting working on 
implementing the distributed barrier. Initial assumptions about port listening/connecting/sending was wrong.
Attempted to first send "ready" messages to all processes on the network, and then listen to receive said messages,
count the number of messages, then send an "all ready" message when the number of ready messages match the number 
of processes. However, sending a message requires connecting to each process, and if each process is sending first,
each process ends up waiting forever :(. Having issues finding a way to do the barrier (send/receive messages with other
processes) on a single thread.

* Idea to implement distributed barrier: Multithread. One thread listens for barrier messages, counting the number of ready
and allready messages. Another thread attempts to send ready messages to all nodes in the network (including itself). It keeps
retrying until it has sent one message to each node. Once the number of ready messages received matches the number of nodes, broadcast
an allready message. The program continues when it has received as many allready messages as there are processes.

* Tested program on aws instance successfully. The distributed barrier appears to be working for a single process.

* Finished initial attempt at client application. It generates the requests based off of the properties file, and sends them to the respective
server one by one, retrying if necessary until it has sent all requests.

* Currently unsure whether it is better to have a persistent connection between each client and server, or only make the connections as needed.
Implemented both versions (unsure if they work), but currently moving forward with remaking connections as needed because the server is currently
not written to accept multiple persistent connections.

* Single-threaded server, metrics recording, client all fully implemented. Through testing (one client and one server locally), 
setting ~30000 operations or less has throughput of around 2000. Around 40000 operations or more, throughout suddenly slows to ~350.
Unsure why this happens. Receiving error "Only one usage of each socket address (protocol/network address/port) is normally permitted."
sometimes when client tries to connect to the server. Eventually succeeds, but goes back and forth between success and this error.

* Figured the weird limiting issue/error mentioned regarding throughput was due to port exhaustion. Each time the client made a connection,
it used a port, which could then not be used for 4 minutes. The default number of usable ports was not enough with higher number of operations,
leading to port exhaustion. By increasing the number of dynamic ports, I was able to get ~2000 throughput with 100000 operations (same environment
as previous bullet point).

* Port exhaustion does not seem to happen in the linux environment. I assume it's because windows and linux handle ports differently.

* Started multithreaded implementation by first doing a very coarse synchronization pattern, basically locking the entire hashtable for each operation.
I also started with simply spawning a thread to handle each request. Will move to making a thread pool and bucket locking.