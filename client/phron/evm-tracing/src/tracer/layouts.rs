
use super::listeners::{ListenerRaw, ListenerCl};

use super::calls::{ExplorerCallInner, CallTracerCall, CallTracerInner};
use super::types::{Call, SingleTransactionTrace};

use super::{CallType, CreateResult};


use sp_std::{cmp::Ordering, vec::Vec};


pub struct RawLayout;
impl super::ResponseLayout for RawLayout {
	type Listener = ListenerRaw;
	type Response = SingleTransactionTrace;

	fn format(listener: ListenerRaw) -> Option<SingleTransactionTrace> {
		if listener.remaining_memory_usage.is_none() {
			None
		} else {
			Some(SingleTransactionTrace::Raw {
				struct_logs: listener.struct_logs,
				gas: listener.final_gas.into(),
				return_value: listener.return_value,
			})
		}
	}
}

pub struct ExplorerLayout;
impl super::ResponseLayout for ExplorerLayout {
	type Listener = ListenerCl;
	type Response = SingleTransactionTrace;

	fn format(listener: ListenerCl) -> Option<SingleTransactionTrace> {
		if let Some(entry) = listener.entries.last() {
			return Some(SingleTransactionTrace::CallList(
				entry
					.into_iter()
					.map(|(_, value)| Call::Explorer(value.clone()))
					.collect(),
			));
		}
		None
	}
}

pub struct CallTracerLayout;
impl super::ResponseLayout for CallTracerLayout {
	type Listener = ListenerCl;
	type Response = Vec<SingleTransactionTrace>;

	fn format(mut listener: ListenerCl) -> Option<Vec<SingleTransactionTrace>> {
		// Remove empty BTreeMaps pushed to `entries`.
		// I.e. InvalidNonce or other pallet_evm::runner exits
		listener.entries.retain(|x| !x.is_empty());
		let mut traces = Vec::new();
		for entry in listener.entries.iter() {
			let mut result: Vec<Call> = entry
				.into_iter()
				.map(|(_, it)| {
					let from = it.from;
					let trace_address = it.trace_address.clone();
					let value = it.value;
					let gas = it.gas;
					let gas_used = it.gas_used;
					let inner = it.inner.clone();
					Call::CallTracer(CallTracerCall {
						from,
						gas,
						gas_used,
						trace_address: Some(trace_address.clone()),
						inner: match inner.clone() {
							ExplorerCallInner::Call {
								input,
								to,
								res,
								call_type,
							} => CallTracerInner::Call {
								call_type: match call_type {
									CallType::Call => "CALL".as_bytes().to_vec(),
									CallType::CallCode => "CALLCODE".as_bytes().to_vec(),
									CallType::DelegateCall => "DELEGATECALL".as_bytes().to_vec(),
									CallType::StaticCall => "STATICCALL".as_bytes().to_vec(),
								},
								to,
								input,
								res,
								value: Some(value),
							},
							ExplorerCallInner::Create { init, res } => CallTracerInner::Create {
								input: init,
								error: match res {
									CreateResult::Success { .. } => None,
									CreateResult::Error { ref error } => Some(error.clone()),
								},
								to: match res {
									CreateResult::Success {
										created_contract_address_hash,
										..
									} => Some(created_contract_address_hash),
									CreateResult::Error { .. } => None,
								},
								output: match res {
									CreateResult::Success {
										created_contract_code,
										..
									} => Some(created_contract_code),
									CreateResult::Error { .. } => None,
								},
								value,
								call_type: "CREATE".as_bytes().to_vec(),
							},
							ExplorerCallInner::SelfDestruct { balance, to } => {
								CallTracerInner::SelfDestruct {
									value: balance,
									to,
									call_type: "SELFDESTRUCT".as_bytes().to_vec(),
								}
							}
						},
						calls: Vec::new(),
					})
				})
				.collect();
			if result.len() > 1 {
				result.sort_by(|a, b| match (a, b) {
					(
						Call::CallTracer(CallTracerCall {
							trace_address: Some(a),
							..
						}),
						Call::CallTracer(CallTracerCall {
							trace_address: Some(b),
							..
						}),
					) => {
						let a_len = a.len();
						let b_len = b.len();
						let sibling_greater_than = |a: &Vec<u32>, b: &Vec<u32>| -> bool {
							for (i, a_value) in a.iter().enumerate() {
								if a_value > &b[i] {
									return true;
								} else if a_value < &b[i] {
									return false;
								} else {
									continue;
								}
							}
							return false;
						};
						if b_len > a_len || (a_len == b_len && sibling_greater_than(&a, &b)) {
							Ordering::Less
						} else {
							Ordering::Greater
						}
					}
					_ => unreachable!(),
				});
				// Stack pop-and-push.
				while result.len() > 1 {
					let mut last = result
						.pop()
						.expect("result.len() > 1, so pop() necessarily returns an element");
					// Find the parent index.
					if let Some(index) =
						result
							.iter()
							.position(|current| match (last.clone(), current) {
								(
									Call::CallTracer(CallTracerCall {
										trace_address: Some(a),
										..
									}),
									Call::CallTracer(CallTracerCall {
										trace_address: Some(b),
										..
									}),
								) => {
									&b[..]
										== a.get(0..a.len() - 1).expect(
											"non-root element while traversing trace result",
										)
								}
								_ => unreachable!(),
							}) {
						// Remove `trace_address` from result.
						if let Call::CallTracer(CallTracerCall {
							ref mut trace_address,
							..
						}) = last
						{
							*trace_address = None;
						}
						// Push the children to parent.
						if let Some(Call::CallTracer(CallTracerCall { calls, .. })) =
							result.get_mut(index)
						{
							calls.push(last);
						}
					}
				}
			}
			// Remove `trace_address` from result.
			if let Some(Call::CallTracer(CallTracerCall { trace_address, .. })) = result.get_mut(0)
			{
				*trace_address = None;
			}
			if result.len() == 1 {
				traces.push(SingleTransactionTrace::CallListNested(result.pop().expect(
					"result.len() == 1, so pop() necessarily returns this element",
				)));
			}
		}
		if traces.is_empty() {
			return None;
		}
		return Some(traces);
	}
}

