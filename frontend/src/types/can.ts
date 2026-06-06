// CAN frame types

export interface CanFrame {
  cob_id: number;
  data: number[];
  dlc: number;
  timestamp_ms: number;
  direction: 'tx' | 'rx';
}

export interface BusStats {
  bus_load: number;
  frame_rate: number;
  tx_errors: number;
  rx_errors: number;
  error_frame_count: number;
}

export interface ErrorFrame {
  timestamp_ms: number;
  error_type: string;
  tec: number;
  rec: number;
}
