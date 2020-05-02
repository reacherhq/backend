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

// standard-version-updater.js
// https://github.com/conventional-changelog/standard-version

module.exports.readVersion = function (contents) {
	console.log(contents);
};

module.exports.writeVersion = function (contents, version) {
	const json = JSON.parse(contents);
	json.tracker.package.version = version;

	return stringifyPackage(json, indent, newline);
};
