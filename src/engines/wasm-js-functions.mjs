export function sendUpdates(tick, pendingEvents, updates) {
    let updatate = [];
    let i = 0;
    while (i < updates.length) {
        let ups = updates[i];
        let outputs = {}
        let j = 0
        while (j < ups.val.length) {
            let ports = ups.val[j];
            outputs[ports.port] = { _bits: ports.bits, _avec: ports.avec, _bvec: ports.bvec };
            j++;
        }
        updatate.push([ups.graph_id, ups.gate_id, outputs])
        i++;
    }
    postMessage({type: 'update', args: [tick, pendingEvents, updatate]});
}

export function triggerMemoryUpdate(graphId, gateId, address, bits, avec, bvec) {
    postMessage({ type: 'gateTrigger', args: [graphId, gateId, 'memChange', [address, { _bits: bits, _avec: avec, _bvec: bvec }]] });
}