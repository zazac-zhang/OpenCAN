// DS402 state types

export interface Ds402NodeState {
  node_id: number;
  state: string;
  status_word: number;
  control_word?: number;
  actual_position: number;
  actual_velocity: number;
  actual_torque: number;
  selected_mode: string;
  target_position: string;
  target_velocity: string;
  target_torque: string;
  auto_refresh: boolean;
  raw_values: boolean;
  position_history: DataPoint[];
  velocity_history: DataPoint[];
  torque_history: DataPoint[];
}

export interface DataPoint {
  time: number;
  value: number;
}
