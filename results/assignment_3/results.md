# Results
##### Overview
* DHT was run on 5 AWS EC2 instances. The instance type was t2.xlarge, which has 4 vCPUs, 16 GiB memory and "Moderate" 
    (~300-900Mb/s) network performance. All instances were hosted in the same region, us-east-1a. Testing was done with
    5 nodes, 8 client threads per node, and 10000 operations per client thread.
* No modifications necessary from Assignment #2, as discussed in grading meeting.

##### Logging Implementation
* The log for the client is saved in log/client.log
* The log for the server is saved in log/server.log
* Logging is only done for Put and MultiPut requests since only those two use 2PC
* Client logging in client/client_application.rs
    * Prepare to commit: line 164 (Put) and line 236 (MultiPut)
    * Commit: line 201 (Put) and line 285 (MultiPut)
    * Abort: line 221 (Put) and line 306 (MultiPut)
* Server logging in server/server_functions.rs
    * Agree to commit: line 132 (Put) and line 27 (MultiPut)
    * Abort: line 170 (Put) and line 72 (MultiPut)
    * Commit: line 146 (Put) and line 37 (MultiPut)

##### Charts
![Latency](DHT_Average_Latency_with_5_Nodes.png)

![Throughput](DHT_Average_Throughput_with_5_Nodes.png)

![Failed Requests](DHT_Average_Failed_Requests_with_5_Nodes.png)