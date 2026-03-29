import { invoke } from '@tauri-apps/api/core';

export type PlayerToAct = 'oop' | 'ip';

export interface RiverSolveRequest {
  board: string;
  oopRange: string;
  ipRange: string;
  potSize: string;
  oopStack: string;
  ipStack: string;
  playerToAct: PlayerToAct;
  smallBlind: string;
  bigBlind: string;
  firstBetSize: string;
  afterCheckBetSize: string;
  iterations: string;
  deterministicSeed: string;
}

export interface AppError {
  code: string;
  message: string;
}

export interface ActionProbability {
  label: string;
  probability: number;
}

export interface InfosetStrategy {
  key: string;
  actions: ActionProbability[];
}

export interface NormalizedRiverConfig {
  schemaVersion: number;
  variant: string;
  mode: string;
  activePlayers: number;
  street: string;
  board: string;
  oopRange: string;
  ipRange: string;
  potSize: number;
  oopStack: number;
  ipStack: number;
  playerToAct: string;
  smallBlind: number;
  bigBlind: number;
  rakeBps: number;
  rakeCap: number;
  templateId: string;
  templateVersion: number;
  firstBetSize: number;
  afterCheckBetSize: number;
  maxRaisesPerStreet: number;
  allowAllIn: boolean;
  algorithm: string;
  iterations: number;
  checkpointCadence: number;
  threadCount: number;
  deterministicSeed: string;
}

export interface ValidateConfigResponse {
  configHash: string;
  compatibleDealCount: number;
  normalized: NormalizedRiverConfig;
}

export interface RiverSolveResponse {
  configHash: string;
  compatibleDealCount: number;
  normalized: NormalizedRiverConfig;
  treeIdentity: string;
  iterations: number;
  rootValueOop: number;
  nashConv: number;
  p0Improvement: number;
  p1Improvement: number;
  rootInfosets: InfosetStrategy[];
}

export const emptyRiverSolveRequest = (): RiverSolveRequest => ({
  board: '',
  oopRange: '',
  ipRange: '',
  potSize: '0',
  oopStack: '100',
  ipStack: '100',
  playerToAct: 'oop',
  smallBlind: '1',
  bigBlind: '2',
  firstBetSize: '0',
  afterCheckBetSize: '0',
  iterations: '1000',
  deterministicSeed: '0',
});

function normalizeError(error: unknown): AppError {
  if (error && typeof error === 'object' && 'code' in error && 'message' in error) {
    return error as AppError;
  }
  if (typeof error === 'string') {
    return { code: 'unknown', message: error };
  }
  return { code: 'unknown', message: 'Unknown desktop command error' };
}

export async function loadSampleRequest(): Promise<RiverSolveRequest> {
  try {
    return await invoke<RiverSolveRequest>('sample_river_request');
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function validateConfig(request: RiverSolveRequest): Promise<ValidateConfigResponse> {
  try {
    return await invoke<ValidateConfigResponse>('validate_config', { request });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function solveRiverSpot(request: RiverSolveRequest): Promise<RiverSolveResponse> {
  try {
    return await invoke<RiverSolveResponse>('solve_river_spot', { request });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function writeE2eSmokeReport(report: Record<string, unknown>): Promise<void> {
  try {
    await invoke('write_e2e_smoke_report', { report: JSON.stringify(report) });
  } catch (error) {
    throw normalizeError(error);
  }
}
