# DistributedHashTable (DHT)

### About
* Project for CSE 403 (Advanced Operating Systems)
* Author: Dan Yu (Student at Lehigh University)

##### Description:
* This project creates a distributed hash table. The two main parts are the client application
and the server. You can run as many clients and servers as you want (resources allowing), with
settings defined in the src/properties.json file. Clients and servers are linked, where it is 
assumed that each client runs on the same machine as exactly one other server and vice versa.
The client generates random get/put requests and uses a hash function to decide which server
to request from.
* The DHT also is replicated, where each key/value pair is replicated on some number of servers. 
* Two phase locking is used.
* Logging is done such that it is possible for the system to recover from failures/crashes correctly.

##### Specifications:
* Both servers and client use a distributed barrier, where they don't do anything until they 
confirm that all other servers or clients, respectively, are online and ready.
* The server uses a thread pool (number of threads specified in src/properties.json) to handle 
multiple requests at the same time.
* A custom implementation of a hash table is used, where each bucket (number of buckets specified 
in src/properties.json) is a vec of key-value pairs. This was done in order to make the server
have a mutex over both get and put requests, rejecting requests if the lock cannot be acquired.

##### Project Directory:
* [src] contains all the source code of the project
    * [client] contains the source code for the client application
    * [server] contains the source code for the server
    * [common] contains the source code for functions/structs used by both server and client.
    * properties.json contains all the initial setup data for running the DHT
* [results] contains the metrics and analysis of the metrics taken of the DHT

##### Run
* Go to ~/DistributedHashTable
* To run server:
    * cargo run --bin dht_server
* To run client:
    * cargo run --bin client_application
