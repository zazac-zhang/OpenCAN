// CANOpen protocol types

export interface NodeInfo {
  node_id: number;
  nmt_state: string;
  device_type?: number;
  vendor_id?: number;
  product_name?: string;
}

export interface SdoEntry {
  node_id: number;
  index: number;
  subindex: number;
  value: string;
  is_read: boolean;
  success: boolean;
  error?: string;
}

export interface PdoEntry {
  node_id: number;
  pdo_type: 'tpdo' | 'rpdo';
  cob_id: number;
  data: number[];
  timestamp_ms: number;
}

export interface EmcyEntry {
  timestamp_ms: number;
  node_id: number;
  error_code: number;
  error_register: number;
  data: number[];
}

export interface HeartbeatEntry {
  node_id: number;
  alive: boolean;
  last_seen_ms: number;
}

export interface SyncStatus {
  is_producer: boolean;
  producer_period_us: number;
}
