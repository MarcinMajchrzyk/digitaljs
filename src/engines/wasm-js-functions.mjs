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

export function postMonitorValue(monitorId, tick, bits, avec, bvec, stopOnTrigger, oneShot) {
    postMessage({ type: 'monitorValue', args: [monitorId, tick, { _bits: bits, _avec: avec, _bvec: bvec }, stopOnTrigger, oneShot] });
}

export function updater_stop() {
    postMessage({ type: "updater_stop" });
}

export function sendAck(reqid, response) {
    postMessage({ type: 'ack', args: [reqid, response] });
}

export function sendAlarmReached(alarmId, tick, stopOnAlarm) {
    postMessage({ type: 'alarmReached', args: [alarmId, tick, stopOnAlarm] });
}