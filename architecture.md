# Radicle CI Broker Architecture

The below diagram shows the main entities / interactions related to the Radicle CI broker.

```
+---------+                                                                                                             
|         |                                                                                                             
|         |                                                                                                             
|         |                                                                                                             
|         |                                                                                                             
|         |                                                                                                             
| Radicle |            0. Listens for                                                                                   
|  Node   |<--------------`rad node                                     +---------+                                     
|         |                events`                                      |         |                                     
|         |                   |                                         |         |                                     
|         |                   |                                         |         |                                     
|         |                   |                                         |         |                                     
|         |                   |                                         |         |                                     
+---------+                   |                                         |Concourse|                                     
                              |                      +----------------->|HTTP API |                                     
                              |                      |                  |         |                                     
                              |                      |                  |         |                                     
                         +---------+                 |                  |         |                                     
                         |         |                 |                  |         |                                     
                         |         |                 |                  |         |                                     
                         |         |                 |                  +---------+                                     
                         |         |                 |                       |                               +---------+
                         |         |        1. Create pipeline               |                               |         |
                         | Radicle |      2. Trigger pipeline run       Schedules                            |         |
                      +->|CI Broker|------3. Watch for completion            |                               |         |
                      |  |         |                                         |                               |         |
                      |  |         |                                         v                               |         |
                      |  |         |                                    +---------+                          | Radicle |
                      |  |         |                                    |         |            +------------>|  HTTPD  |
                      |  |         |                                    |         |            |             |         |
                      |  +---------+                                    |         |            |             |         |
                      |       |                                         |         |            |             |         |
         4. Add comment to Radicle Patch                                |         |            |             |         |
            with pipeline run status                                    |Concourse|   Clones radicle repo    |         |
                                                                        | Worker  |-----and checks out       +---------+
                                                                        |         |          patch                      
                                                                        |         |                                     
                                                                        |         |                                     
                                                                        |         |                                     
                                                                        |         |                                     
                                                                        +---------+                                     
```