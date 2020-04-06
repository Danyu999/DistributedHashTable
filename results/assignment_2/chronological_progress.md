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
    
* I realized that the locktable is unnecessary as I worked more with it and the hashtable. Instead of making and using
    a locktable, I can lock the locks in the hashtable from outside the hashtable and then do my operations. However,
    I will leave it with the locktable since it's nice to separate the two.
    
* For the MultiPuts, I decided that the client will only send each server the put requests relevant to the server.
    I did this to follow the previous pattern where only my client decided which keys went to which server and the
    server simply handled whatever was sent to it. Also, I don't have a clear way of having a server identify which
    number server it is unless I compare its IP with the list of node IPs.
    
* MultiPut fully implemented on client and server. I did something really interesting with MultiPut, where it is scalable
    to have n number of Put operations per MultiPut, configurable through properties.json.
    
* Currently have a weird issue where the client slows down a lot for some operations, but not others on AWS. Appears to
    work fine locally.
    
* When attempting to run with multiple servers, there are communication errors between clients and servers. Still
    in the process of debugging. UPDATE: Errors were due to improper handling of stream index in the Put
    code client-side. 

* Everything now works as per assignment 2. However, the previously mentioned issue where reads sometimes
    take a really long time (but not always) is still prevalent and prevents me from testing with too
    many operations (since it won't finish for a long time).
    
* Occasionally, the throughput/latency/total_elapsed_time recorded from two different server nodes would be exactly 
    the same. Not sure why/how this happens.