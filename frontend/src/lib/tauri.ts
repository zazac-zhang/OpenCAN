// Tauri API wrappers

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ============ Commands ============

export async function connectBackend(params: {
  backend_type: string;
  channel: string;
  bitrate: number;
  node_id: number;
}) {
  return invoke<BackendInfo>('connect_backend', { params });
}

export async function disconnect() {
  return invoke<void>('disconnect');
}

export async function getBackends() {
  return invoke<BackendDescriptor[]>('get_backends');
}

export async function scanNodes(timeoutMs: number = 3000) {
  return invoke<number[]>('scan_nodes', { timeout_ms: timeoutMs });
}

export async function nmtCommand(nodeId: number, command: string) {
  return invoke<void>('nmt_command', { params: { node_id: nodeId, command } });
}

export async function sdoUpload(params: {
  node_id: number;
  index: number;
  subindex: number;
  data_type: string;
}) {
  return invoke<SdoResult>('sdo_upload', { params });
}

export async function sdoDownload(params: {
  node_id: number;
  index: number;
  subindex: number;
  data: number[];
}) {
  return invoke<void>('sdo_download', { params });
}

export async function ds402Enable(nodeId: number) {
  return invoke<void>('ds402_enable', { node_id: nodeId });
}

export async function ds402FaultReset(nodeId: number) {
  return invoke<void>('ds402_fault_reset', { node_id: nodeId });
}

export async function ds402SetMode(nodeId: number, mode: number) {
  return invoke<void>('ds402_set_mode', { node_id: nodeId, mode });
}

export async function ds402SetTarget(nodeId: number, mode: number, target: number) {
  return invoke<void>('ds402_set_target', { node_id: nodeId, mode, target });
}

export async function readPdoMapping(nodeId: number, pdoIndex: number) {
  return invoke<PdoMapping>('read_pdo_mapping', { node_id: nodeId, pdo_index: pdoIndex });
}

export async function startSync(periodUs: number) {
  return invoke<void>('start_sync', { period_us: periodUs });
}

export async function stopSync() {
  return invoke<void>('stop_sync');
}

export async function loadEdsFile(path: string) {
  return invoke<EdsInfo>('load_eds_file', { path });
}

export async function startRecording(path: string) {
  return invoke<void>('start_recording', { path });
}

export async function stopRecording() {
  return invoke<void>('stop_recording');
}

export async function loadRecording(path: string) {
  return invoke<RecordingMeta>('load_recording', { path });
}

export async function startPlayback(speed: number) {
  return invoke<void>('start_playback', { speed });
}

export async function stopPlayback() {
  return invoke<void>('stop_playback');
}

// ============ Event Listeners ============

export function onFrameStream(callback: (event: CanFrameEvent) => void): Promise<UnlistenFn> {
  return listen<CanFrameEvent>('frame_stream', (e) => callback(e.payload));
}

export function onFrameStreamBatch(callback: (events: CanFrameEvent[]) => void): Promise<UnlistenFn> {
  return listen<CanFrameEvent[]>('frame_stream_batch', (e) => callback(e.payload));
}

export function onPdoStream(callback: (event: PdoEvent) => void): Promise<UnlistenFn> {
  return listen<PdoEvent>('pdo_stream', (e) => callback(e.payload));
}

export function onPdoStreamBatch(callback: (events: PdoEvent[]) => void): Promise<UnlistenFn> {
  return listen<PdoEvent[]>('pdo_stream_batch', (e) => callback(e.payload));
}

export function onLogStream(callback: (event: LogEvent) => void): Promise<UnlistenFn> {
  return listen<LogEvent>('log_stream', (e) => callback(e.payload));
}

export function onEmcyStream(callback: (event: EmcyEvent) => void): Promise<UnlistenFn> {
  return listen<EmcyEvent>('emcy_stream', (e) => callback(e.payload));
}

export function onHeartbeatStream(callback: (event: HeartbeatEvent) => void): Promise<UnlistenFn> {
  return listen<HeartbeatEvent>('heartbeat_stream', (e) => callback(e.payload));
}

export function onDs402StateStream(callback: (event: Ds402StateEvent) => void): Promise<UnlistenFn> {
  return listen<Ds402StateEvent>('ds402_state_stream', (e) => callback(e.payload));
}

export function onBusStatsStream(callback: (event: BusStatsEvent) => void): Promise<UnlistenFn> {
  return listen<BusStatsEvent>('bus_stats_stream', (e) => callback(e.payload));
}

// ============ Types ============

export interface BackendInfo {
  backend_type: string;
  channel: string;
  bitrate: number;
  node_id: number;
}

export interface BackendDescriptor {
  name: string;
  backend_type: string;
  available: boolean;
}

export interface SdoResult {
  node_id: number;
  index: number;
  subindex: number;
  data: number[];
  data_type: string;
}

export interface PdoMapping {
  cob_id: number;
  entries: { index: number; subindex: number; bit_length: number }[];
}

export interface EdsInfo {
  product_name: string;
  vendor_id: number;
  product_code: number;
  revision_number: number;
  baud_rate: number;
}

export interface RecordingMeta {
  path: string;
  frame_count: number;
  duration_ms: number;
  start_time: string;
}

export interface CanFrameEvent {
  cob_id: number;
  data: number[];
  dlc: number;
  timestamp_ms: number;
  direction: string;
}

export interface PdoEvent {
  node_id: number;
  pdo_type: string;
  cob_id: number;
  data: number[];
  timestamp_ms: number;
}

export interface LogEvent {
  level: string;
  message: string;
  timestamp_ms: number;
}

export interface EmcyEvent {
  node_id: number;
  error_code: number;
  error_register: number;
  data: number[];
  timestamp_ms: number;
}

export interface HeartbeatEvent {
  node_id: number;
  state: string;
  timestamp_ms: number;
}

export interface Ds402StateEvent {
  node_id: number;
  state: string;
  status_word: number;
  actual_position: number;
  actual_velocity: number;
  actual_torque: number;
}

export interface BusStatsEvent {
  bus_load: number;
  frame_rate: number;
  tx_errors: number;
  rx_errors: number;
}
