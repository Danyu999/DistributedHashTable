# Chronological Progress Log:

* Changed distributed barrier so that clients check that all servers are up and running before starting.
    Server has a thread dedicated to listening for more clients trying to join the network.

* Added persistent connections/streams with ability to have multiple client threads per client_application.
    Each client thread has their own streams to each server. Each server spawns a new thread to handle
    each connection. When client closes a connection, server terminates the associated thread.

* Attempting to make a script. Ran into many issues, including confusing error messages. Currently, I think
    all my errors are due to my script. Having never scripted in bash before, going straight to multi-threaded
    processes that I must ssh into and run commands on may be trickier than expected. I have spent 8+ hours
    on figuring out scripts and the resulting errors I get from them.

* Scripts now work.

* Improved performance by more than 2 times. Major issue was the serializing/deserializing, 
    where serde_json did not buffer when writing to a TCPStream.
    
* Moved metrics to server-side. Implemented the suggested method of having a thread wake up every now and then
    to record the metrics and then go back to sleep. This allows us to see how throughout changes throughout
    the handling of requests.
    
* Implemented a lock table. Starting to implement 2PL.

* Interesting note: I realized that my hash function for choosing which node each key should go to and which bucket
    a key should go into were the same, which means that my performance from assignment 1 was likely limited by this.
    
* Working on implementing 2PL on client-side... 

* Decided to not have servers confirm that the commit was done after receiving a commit message. If the client was able
    to send a Commit message, then it knows that eventually the commit will actually happen on all relevant servers.


* TODO: Use locks in hashtable instead of locktable; Implement MultiPut; Figure out method of choosing servers for
    replication; 