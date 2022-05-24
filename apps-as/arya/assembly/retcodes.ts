// This file is part of TRINCI.
//
// Copyright (C) 2021 Affidaty Spa.
//
// TRINCI is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// TRINCI is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with TRINCI. If not, see <https://www.gnu.org/licenses/>.

import { RetCode } from './types';

export namespace retCodes {
    export const ok = new RetCode(0, '');
    export const noInit = new RetCode(1, 'Not initialized.');
    export const noCert = new RetCode(2, 'No valid certificate found.');
    export const excessFields = new RetCode(3, 'Fields not present within certificate.');
    export const missingData = new RetCode(4, 'Not enough data provided.');
    export const noDeleg = new RetCode(5, 'Delegation not found.');
}