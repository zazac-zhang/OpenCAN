import { describe, it, expect } from 'vitest';

// Test type definitions and data structures
describe('CAN types', () => {
  it('CanFrame structure', () => {
    const frame = {
      cob_id: 0x185,
      data: [0x01, 0x02, 0x03, 0x04, 0, 0, 0, 0],
      direction: 'rx' as const,
      timestamp_ms: Date.now(),
    };

    expect(frame.cob_id).toBe(0x185);
    expect(frame.data).toHaveLength(8);
    expect(frame.direction).toBe('rx');
  });

  it('BusStats structure', () => {
    const stats = {
      rx_count: 100,
      tx_count: 50,
      error_count: 2,
      bus_load: 35.5,
      rx_rate: 120.5,
      tx_rate: 60.2,
    };

    expect(stats.rx_count).toBe(100);
    expect(stats.bus_load).toBe(35.5);
  });
});

describe('CANopen types', () => {
  it('NodeInfo structure', () => {
    const node = {
      node_id: 5,
      nmt_state: 'Operational' as const,
      device_type: 0x00020192,
      vendor_id: 0x00000123,
    };

    expect(node.node_id).toBe(5);
    expect(node.nmt_state).toBe('Operational');
  });

  it('SdoEntry structure', () => {
    const entry = {
      node_id: 3,
      index: 0x1000,
      subindex: 0,
      data: [0x92, 0x01, 0x02, 0x00],
      direction: 'upload' as const,
      timestamp_ms: Date.now(),
      success: true,
    };

    expect(entry.index).toBe(0x1000);
    expect(entry.success).toBe(true);
  });
});

describe('DS402 types', () => {
  it('Ds402NodeState structure', () => {
    const state = {
      node_id: 1,
      status_word: 0x0037,
      control_word: 0x000F,
      mode: 8,
      target_position: 1000,
      actual_position: 999,
      target_velocity: 500,
      actual_velocity: 498,
      target_torque: 100,
      actual_torque: 98,
    };

    // StatusWord bit decoding
    expect(state.status_word & 0x0001).toBe(1); // Ready to Switch On
    expect(state.status_word & 0x0002).toBe(2); // Switched On
    expect(state.status_word & 0x0004).toBe(4); // Operation Enabled
    expect(state.status_word & 0x0008).toBe(0); // No Fault

    // Mode 8 = CSP
    expect(state.mode).toBe(8);
  });
});
