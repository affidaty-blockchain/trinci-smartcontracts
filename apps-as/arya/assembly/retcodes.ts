import { RetCode } from './types';

export namespace retCodes {
    export const ok = new RetCode(0, '');
    export const noInit = new RetCode(1, 'Not initialized');
    export const noCert = new RetCode(2, 'Certificate nof found');
    export const excessFields = new RetCode(3, 'Fields not present within certificate');
    export const missingData = new RetCode(4, 'Not enough data provided');
}