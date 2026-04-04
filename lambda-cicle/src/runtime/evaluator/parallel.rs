use super::{verify_s5_prime, EvalError, Evaluator};
use crate::core::ast::{Literal, Term};
use crate::runtime::evaluator::sequential::ExtractionError;
use crate::runtime::net::{Agent, Net, NodeId, PortIndex};
use crossbeam::queue::SegQueue;
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};

pub struct ParallelEvaluator {
    num_threads: usize,
    max_steps: usize,
}

impl ParallelEvaluator {
    pub fn new() -> ParallelEvaluator {
        ParallelEvaluator {
            num_threads: num_cpus::get(),
            max_steps: 10_000_000,
        }
    }

    pub fn with_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }
}

impl Default for ParallelEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for ParallelEvaluator {
    fn evaluate(&self, net: &mut Net) -> Result<Term, EvalError> {
        verify_s5_prime(net)?;

        let net = Arc::new(Mutex::new(std::mem::take(net)));
        let work_queue: Arc<SegQueue<WorkItem>> = Arc::new(SegQueue::new());
        let shutdown = Arc::new(Mutex::new(false));
        let work_done = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let parallel_subgraphs = find_parallel_subgraphs(&net.lock());
        for (id1, id2) in parallel_subgraphs {
            work_queue.push(WorkItem::Parallel(id1, id2));
        }

        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for _ in 0..self.num_threads {
            let net = Arc::clone(&net);
            let queue = Arc::clone(&work_queue);
            let shutdown = Arc::clone(&shutdown);
            let work_done = Arc::clone(&work_done);
            let max_steps = self.max_steps;

            let handle = spawn(move || {
                worker_loop(&net, &queue, &shutdown, &work_done, max_steps);
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        let result = {
            let net_guard = net.lock();
            extract_result(&net_guard)
        };

        let _final_net = Arc::try_unwrap(net)
            .map(|m| m.into_inner())
            .unwrap_or_else(|_| Net::new());

        result.map_err(|err| EvalError::EvaluationError(format!("{:?}", err)))
    }
}

fn worker_loop(
    net: &Arc<Mutex<Net>>,
    queue: &Arc<SegQueue<WorkItem>>,
    shutdown: &Arc<Mutex<bool>>,
    work_done: &Arc<std::sync::atomic::AtomicUsize>,
    max_steps: usize,
) {
    let mut steps = 0;

    loop {
        {
            let is_shutdown = *shutdown.lock();
            if is_shutdown && queue.is_empty() {
                break;
            }
        }

        let work = queue.pop().or_else(|| steal_from(queue));

        match work {
            Some(WorkItem::Parallel(id1, id2)) => {
                {
                    let net_guard = net.lock();
                    if let Some((n1, _)) = net_guard.get_connected_port(id1, PortIndex(1)) {
                        queue.push(WorkItem::Reduce(n1));
                    }
                    if let Some((n2, _)) = net_guard.get_connected_port(id2, PortIndex(2)) {
                        queue.push(WorkItem::Reduce(n2));
                    }
                }
                work_done.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            Some(WorkItem::Reduce(_node_id)) => {
                {
                    let mut net_guard = net.lock();
                    net_guard.step();
                    steps += 1;
                }
                work_done.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                if steps >= max_steps {
                    *shutdown.lock() = true;
                    break;
                }

                {
                    let net_guard = net.lock();
                    if net_guard.is_stuck() {
                        *shutdown.lock() = true;
                    }
                }
            }
            None => {
                std::thread::yield_now();
            }
        }
    }
}

fn steal_from(queue: &Arc<SegQueue<WorkItem>>) -> Option<WorkItem> {
    for _ in 0..10 {
        if let Some(item) = queue.pop() {
            return Some(item);
        }
        std::thread::yield_now();
    }
    None
}

fn find_parallel_subgraphs(net: &Net) -> Vec<(NodeId, NodeId)> {
    let mut subgraphs = Vec::new();

    for (delta_id, node) in net.nodes().iter().enumerate() {
        if !matches!(node.agent, Agent::Delta) {
            continue;
        }

        let delta_id = NodeId(delta_id);

        let output1 = net.get_connected_port(delta_id, PortIndex(1));
        let output2 = net.get_connected_port(delta_id, PortIndex(2));

        if let (Some((target1, _)), Some((target2, _))) = (output1, output2) {
            subgraphs.push((target1, target2));
        }
    }

    subgraphs
}

fn extract_result(net: &Net) -> Result<Term, ExtractionError> {
    if net.nodes().is_empty() && net.wires().is_empty() {
        return Ok(Term::NativeLiteral(Literal::Unit));
    }

    for (id, node) in net.nodes().iter().enumerate() {
        if matches!(node.agent, Agent::Constructor(_) | Agent::Prim(_)) {
            let mut has_free_ports = false;
            for port_idx in 0..node.num_ports() {
                if net
                    .get_connected_port(NodeId(id), PortIndex(port_idx))
                    .is_none()
                {
                    has_free_ports = true;
                    break;
                }
            }
            if !has_free_ports && node.num_ports() == 0 {
                match &node.agent {
                    Agent::Constructor(name) => {
                        return Ok(Term::Var(name.clone()));
                    }
                    Agent::Prim(_) => {
                        return Ok(Term::NativeLiteral(Literal::Unit));
                    }
                    _ => {}
                }
            }
        }
    }

    Err(ExtractionError::NoResult)
}

#[derive(Debug, Clone)]
enum WorkItem {
    Parallel(NodeId, NodeId),
    Reduce(NodeId),
}

mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
    }
}
