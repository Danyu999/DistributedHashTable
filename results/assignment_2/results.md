# Results
##### Overview
* DHT was run on 5 AWS EC2 instances. The instance type was t2.xlarge, which has 4 vCPUs, 16 GiB memory and "Moderate" 
    (~300-900Mb/s) network performance. All instances were hosted in the same region, us-east-1a.

##### Observations/Notes
* Should failed requests be counted as an operation and be included in the calculations for throughput and latency?
    I did, but that ended up making the runs with smaller key ranges have very good throughput/latency relative to
    the larger key ranges because there are more collisions and failed requests. Failed requests tend to take a lot
    less time server-side, which boosted the throughput/latency numbers. If failed requests were not counted, I suspect
    the graphs would look like mirror images of themselves.
    
* Implemented MultiPut in a scalable way. You can modify how many puts you want to do in one multi-put in the properties
    file. Interesting to implement, ended up using a recursive function to acquire the necessary locks.
    
* Implemented metrics measuring as mentioned in class, where we have a thread that wakes up every now and then to record,
    and then goes back to sleep. This wasn't graphed, but during testing, I noticed that throughput would be lower during
    startup and gradually speed up as the test went on.
    
* Persistent connections was implemented, which made a huge change in performance for the assignment 1 version of the DHT.
    Performance increase by ~10 times.

* I had a weird bug where locally on Windows, the DHT's networking speeds were consistent
    and fine. However, on the linux instances on AWS, the reading from a stream would sometimes,
    not always, take extremely long (~40000 microseconds). Due to the structure of the DHT,
    server throughput was not affected, but communication of operations between client and server
    would take very long, making executing many operations very time consuming.

##### Charts
![Latency](DHT_Average_Latency_with_5_Nodes.png)

![Throughput](DHT_Average_Throughput_with_5_Nodes.png)