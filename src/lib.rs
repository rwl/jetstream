// Copyright (c) 2021 Synaptec Ltd
// Copyright (c) 2022 Richard Lincoln
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program.
// If not, see <https://www.gnu.org/licenses/>.
mod decoder;
pub mod emulator;
mod encoder;
pub mod encoding;
mod jetstream;
#[cfg(test)]
mod test;
pub mod testcase;

pub use crate::decoder::Decoder;
pub use crate::encoder::Encoder;
pub use crate::jetstream::*;
