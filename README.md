# WASM interpreter

Just me trying to fully comprehend the wasm spec.

## Durable Execution

The goal of the project is to build an WASM interpreter whose execution is suspendable and durable. The idea originated when I was trying to implement a framework for long running workflows.

* It's a WASM runtime, so we can run anything as a workflow given it can be targetd to WASM.
* It's durable, so it can survive crashes. And more importantly, service upgrades, you always have tasks running if they are long run tasks.

The usual approach of allow such tasks is to invent custom DSLs or to be more specific, explicitly define intermediate states so it can be persisted in a message queue or a database.
And the problem with this approach is most efforts goes into maintaining custom DSL and states and the infrastructures running it instead of the actual business to support.

Airflow is good as a taskflow engine but it's DAG based, which provides it potential parallelizm, but limits the dynamicity of the flow.

## The runtime

The major strugling point back when I was doing the design work is how to run these things. 
We have kafka, but not sure if it's a good idea using a single topic and trampoline the execution.

It's Ok to just run the programs, but when it comes to communicate with the external world:
1. submit task
2. fan out tasks to worker nodes (the workflow emits concrete heavy tasks to run on worker nodes)
3. resume the flow when task run to a result on the worker node

Flows are sharded on different nodes, so a typical cluster of services behind an LB won't work directly.

After studying how people architect distributed systems. I think I have finally come to a rough working solution.

That's to store versioned sharding topology in raft. And having 2 clusters: 1. the workflow nodes run the workflow. 2. the API nodes watch tasks and dispatch task results to the belonging node.

### Forwarder

Amazon Aurora added write forwarder capability since a version. I think we can use the same technique to eliminate the need of API nodes dual functioning as an API service and forward requests to the responsible node.
