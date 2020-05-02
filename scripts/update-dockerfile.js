// Reacher
// Copyright (C) 2018-2020 Amaury Martiny

// Reacher is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Reacher is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Reacher.  If not, see <http://www.gnu.org/licenses/>.

// Update the version in `Dockerfile` each time we run `standard-version`.
// https://github.com/conventional-changelog/standard-version

module.exports.readVersion = function (contents) {
	// Find the version in the Dockerfile
	return /[0-9]+\.[0-9]+\.[0-9]+/.exec(contents)[0];
};

module.exports.writeVersion = function (contents, newVersion) {
	const oldVersion = /[0-9]+\.[0-9]+\.[0-9]+/.exec(contents)[0];

	// Update version in Dockerfile to the one from standard-version
	return contents.replace(oldVersion, newVersion);
};
