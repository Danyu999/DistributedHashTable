# Chronological Progress Log:

* Changed distributed barrier so that clients check that all servers are up and running before starting.
    Server has a thread dedicated to listening for more clients trying to join the network.

* Added persistent connections/streams with ability to have multiple client threads per client_application.
    Each client thread has their own streams to each server. Each server spawns a new thread to handle
    each connection. When client closes a connection, server terminates the associated thread.

* 