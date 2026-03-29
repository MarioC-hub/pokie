import { invoke } from '@tauri-apps/api/core';

export type PlayerToAct = 'oop' | 'ip';

export interface RiverSolveRequest {
  board: string;
  oopRange: string;
  ipRange: string;
  potSize: number;
  oopStack: number;
  ipStack: number;
  playerToAct: PlayerToAct;
  smallBlind: number;
  bigBlind: number;
  firstBetSize: number;
  afterCheckBetSize: number;
  iterations: number;
  deterministicSeed: number;
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
  player: string;
  privateHand: string;
  history: string;
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
  deterministicSeed: number;
}

export interface ValidateConfigResponse {
  configHash: string;
  compatibleDealCount: number;
  normalized: NormalizedRiverConfig;
}

export interface RiverSolveResponse {
  configHash: string;
  treeIdentity: string;
  iterations: number;
  rootValueOop: number;
  nashConv: number;
  p0Improvement: number;
  p1Improvement: number;
  rootInfosets: InfosetStrategy[];
}

export const fallbackSampleRequest: RiverSolveRequest = {
  board: 'Ks7d4c2h2d',
  oopRange: '7c7h:1.0,AcJc:1.0',
  ipRange: 'KcQh:1.0',
  potSize: 10,
  oopStack: 100,
  ipStack: 100,
  playerToAct: 'oop',
  smallBlind: 1,
  bigBlind: 2,
  firstBetSize: 10,
  afterCheckBetSize: 0,
  iterations: 2000,
  deterministicSeed: 0,
};

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
  } catch {
    return fallbackSampleRequest;
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
