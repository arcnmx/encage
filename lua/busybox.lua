--
-- Helper Methods
--

local print_r
print_r = function(data, indent)
	local io = io.stderr
	if indent == nil then
		indent = 0
	end
	if type(data) == "table" then
		for k, v in pairs(data) do
			for i = 0, indent do
				io:write("--")
			end
			io:write(string.format("%s", k) .. "\n")
			print_r(v, indent + 1)
		end
	else
		for i=0,indent do
			io:write("--")
		end
		io:write(string.format("%s", data) .. "\n")
	end
end

local array_or_single = function(data, field)
	local value = data[field]
	if value == nil then
		return nil
	end

	if value._type == nil then
		return data
	end

	data[field] = {value}
end

local array_flatten
array_flatten = function(...)
	local ret = {}

	local args = {...}
	for i = 1, #args do
		if type(args[i]) == "table" then
			for _, v in ipairs(array_flatten(unpack(args[i]))) do
				table.insert(ret, v)
			end
		else
			table.insert(ret, args[i])
		end
	end

	return ret
end

local expand = function(s, data)
	local gsplit = function(s,sep)
		local lasti, done, g = 1, false, s:gmatch('(.-)'..sep..'()')
		return function()
			if done then return end
			local v,i = g()
			if s == '' or sep == '' then done = true return s end
			if v == nil then done = true return s:sub(lasti) end
			lasti = i
			return v
		end
	end

	local replace = function(w)
		local request = w:sub(3, -2)
		local params = {}
		for ident in gsplit(request, " ") do
			table.insert(params, ident)
		end

		local query = data
		local request = table.remove(params, 1)
		for ident in gsplit(request, "%.") do
			query = query[ident]
			if query == nil then
				break
			end
		end

		if #params > 0 and query ~= nil then
			query = query(unpack(params))
		end

		return query or w
	end
	return s:gsub('(%%%b{})', replace)
end

local expand_replace
expand_replace = function(table, data)
	local shallow_clone = function(table)
		if table == nil then
			return {}
		end

		local out = {}
		for k, v in pairs(table) do
			out[k] = v
		end
		return out
	end

	data = shallow_clone(data)
	if table.vars ~= nil then
		data.vars = shallow_clone(data.vars)
		for k, v in pairs(table.vars) do
			data.vars[k] = v
		end
	end

	for k, v in pairs(table) do
		if type(v) == "string" then
			table[k] = expand(v, data)
		elseif type(v) == "table" then
			expand_replace(v, data)
		end
	end
end

local assert_type = function(data, ty)
	if data ~= nil and data._type ~= ty then
		error("Expected type " .. ty)
	end
end

local assert_type_array = function(data, ty)
	if data ~= nil then
		for k, v in pairs(data) do
			assert_type(v, ty)
		end
	end
end

local hash
hash = function(data)
	local value = 0
	local odata = {}
	if type(data) == "table" then
		for k, v in pairs(data) do
			table.insert(odata, bit32.bxor(bit32.lrotate(hash(k), 15), hash(v)))
		end
		table.sort(odata)
		for i, v in ipairs(odata) do
			value = bit32.bxor(value, bit32.lrotate(v, i))
		end
	elseif type(data) == "string" then
		for i = 1, #data do
			local c = string.byte(data, i)
			value = bit32.lrotate(bit32.bxor(value, c), i - 1)
		end
	elseif type(data) == "number" then
		value = bit32.bnot(data)
	elseif type(data) == "boolean" then
		if data then
			value = 1
		else
			value = 2
		end
	elseif type(data) == "nil" then
		value = 3
	end

	return value
end

local hash_str = function(data)
	return string.format("%08x", hash(data))
end

--
-- BuildDAG
--

local DAG = {}
function DAG:new()
	local dag = {}
	setmetatable(dag, self)
	self.__index = self
	return dag
end

function DAG:add(output, commands, inputs)
	if inputs == nil then
		inputs = {}
	elseif type(inputs) ~= "table" then
		inputs = {inputs}
	end

	if commands == nil then
		commands = {}
	elseif commands._type == "encage.command" then
		commands = {commands}
	end

	self[output] = {
		commands = commands,
		inputs = inputs
	}
end

function DAG:dummy(output)
	self:add(
		output,
		Command {
			desc = "[touch] " .. output,
			"touch", output
		}
	)
end

function DAG:phony(name, input)
	self:add(
		name,
		nil,
		input
	)
end

function DAG:depend(output, ...)
	for _, input in ipairs({...}) do
		table.insert(self[output].inputs, input)
	end
end

function DAG:write_ninja(fd)
	local ninja_escape = function(str)
		return str:gsub("%$", "$$"):gsub("\n", "$\n")
	end

	for output, v in pairs(self) do
		local rule_name
		if #v.commands == 0 then
			rule_name = "phony"
		else
			rule_name = hash_str({ output, v })

			local command = ""
			for _, cmd in ipairs(v.commands) do
				if #command > 0 then
					command = command .. " && "
				end
				command = command .. cmd:shellish_string()
			end

			fd:write("rule " .. rule_name .. "\n")
			fd:write("    command = " .. ninja_escape(command) .. "\n")

			local desc = v.commands.desc
			if desc == nil and #v.commands > 0 then
				desc = v.commands[1].desc
			end
			if desc ~= nil then
				fd:write("    description = " .. ninja_escape(desc) .. "\n")
			end
		end

		fd:write("build " .. output .. ": " .. rule_name)
		for _, input in ipairs(v.inputs) do
			fd:write(" " .. input)
		end
		fd:write("\n")
	end
end


--
-- Encage
--

function Encage(package)
	-- print_r(package)

	local packages = {}

	for _, image in pairs(package.images) do
		packages[image.name] = image

		local files = {}
		if image.files ~= nil then
			for _, file in pairs(image.files) do
				files[file.dest] = hash_str(file.src) .. "/" .. file:file_name()
			end
		end

		expand_replace(image, {
			path = {
				build = "build",
				target = "/mnt/target",
				root = "/",
				file = function(filename)
					return "/mnt/res/" .. files[filename]
				end
			},
		})
	end

	local output_path = function(image, name)
		return "build/" .. image.name .. "/" .. name
	end

	local stage_path = function(image, name)
		return output_path(image, "stage/" .. name)
	end

	local stamp_path = function(image, stamp)
		return output_path(image, "stamp/" .. stamp)
	end

	local command_download = function(file, output)
		local url = file.src
		local desc = "[download] " .. url
		if file.download_cont then
			return Command {
				desc = desc,
				"curl", "-C", "-", "-Lo", output, url
			}
		else
			return Command {
				desc = desc,
				"curl", "-Lo", output, url
			}
		end
	end

	local command_mkdir = function(path, stamp)
		return {
			desc = "[mkdir] " .. path,
			Command { "mkdir", "-p", path },
			Command { "touch", stamp },
		}
	end

	local command_copy = function(path, output)
		return Command {
			desc = "[copy] " .. path,
			"cp", "-a", path, output
		}
	end

	local command_chmod = function(stamp, path, data)
		local commands = {
			desc = "[chmod] " .. path,
		}

		if data.mode ~= nil then
			table.insert(commands, Command {
				"chmod",
				string.format("%o", data.mode),
				path,
			})
		end

		table.insert(commands, Command { "touch", stamp })

		return commands
	end

	local image_workdir = function(image, id)
		return output_path(image, "workdir/" .. hash_str(id))
	end

	local image_root
	image_root = function(image)
		image = packages[image.name]
		local mounts = {}

		if image.depends ~= nil then
			for _, v in ipairs(image.depends) do
				for _, mount in ipairs(image_root(v)) do
					table.insert(mounts, mount)
				end
			end
		end
		table.insert(mounts, stage_path(image, "."))

		return mounts
	end

	local image_mounts
	image_mounts = function(image)
		image = packages[image.name]
		local mounts = {}

		if image.depends ~= nil then
			for _, v in ipairs(image.depends) do
				for _, mount in ipairs(image_mounts(v)) do
					table.insert(mounts, mount)
				end
			end
		end

		if image.mounts ~= nil then
			for _, mount in ipairs(image.mounts) do
				if mount.kind == "bind" then
					table.insert(mounts, "--bind")

					local source = mount.source
					if type(source) == "string" then
						source = {source}
					end

					local bind = mount.target
					for _, v in ipairs(source) do
						bind = v .. ":" .. bind
					end

					local options = mount.options
					if options ~= nil then
						if type(options) == "string" then
							options = {options}
						end

						for _, v in ipairs(options) do
							bind = bind .. "," .. v
						end
					end

					bind = bind .. ",workdir=" .. image_workdir(image, mount)

					table.insert(mounts, bind)
				else
					error("unsupported mount type")
				end
			end
		end

		return mounts
	end

	local image_root_str = function(image)
		local mounts = ""
		for _, v in ipairs(image_root(image)) do
			if #mounts > 0 then
				mounts = mounts .. ":"
			end
			mounts = mounts .. v
		end

		return mounts
	end

	local command_build = function(image, build, cmd, stamp)
		local runcmd = array_flatten {
			"encage-run",
			"exec",
			"--bind", image_root_str(image) .. ":/mnt/target,rw,workdir=" .. image_workdir(image),
			"--bind", output_path(image, "res") .. ":/mnt/res,ro",
			image_mounts(build),
			"--",
			image_root_str(build) .. ",ro,workdir=" .. image_workdir(build),
		}

		for _, arg in ipairs(cmd) do
			table.insert(runcmd, arg)
		end

		local desc = cmd:display_string()
		if cmd.desc ~= nil then
			desc = cmd.desc
		end

		local name = "[" .. image.name .. "]"

		return {
			desc = name .. "[build] " .. desc,
			Command(runcmd),
			Command { "touch", stamp }
		}
	end

	local command_command = function(image, cmd, stamp)
		local runcmd = array_flatten {
			"encage-run",
			"exec",
			"--bind", output_path(image, "res") .. ":/mnt/res,ro",
			image_mounts(image),
			"--",
			image_root_str(image) .. ",workdir=" .. image_workdir(image)
		}

		for _, arg in ipairs(cmd) do
			table.insert(runcmd, arg)
		end

		local desc = cmd:display_string()
		if cmd.desc ~= nil then
			desc = cmd.desc
		end

		local name = "[" .. image.name .. "]"

		return {
			desc = name .. " [command] " .. desc,
			Command(runcmd),
			Command { "touch", stamp }
		}
	end

	local dag = DAG:new()
	for _, image in pairs(package.images) do
		local stamp = {
			image = stamp_path(image, "image"),
			stage = stamp_path(image, "stage"),
			prebuild = stamp_path(image, "prebuild"),
			build = stamp_path(image, "build"),
		}
		dag:dummy(stamp.image)
		dag:dummy(stamp.stage)
		dag:dummy(stamp.prebuild)
		dag:dummy(stamp.build)

		dag:depend(stamp.prebuild, stamp.stage)
		dag:depend(stamp.build, stamp.prebuild)
		dag:depend(stamp.image, stamp.build)

		dag:add(stamp_path(image, "dir.stage"), command_mkdir(output_path(image, "stage"), stamp_path(image, "dir.stage")))
		dag:add(stamp_path(image, "dir.res"), command_mkdir(output_path(image, "res"), stamp_path(image, "dir.res")))

		dag:depend(stamp.stage, stamp_path(image, "dir.stage"))
		dag:depend(stamp.stage, stamp_path(image, "dir.res"))

		dag:phony(image.name, stamp.image)

		if image.depends ~= nil then
			for _, dep in pairs(image.depends) do
				dag:depend(stamp.stage, stamp_path(dep, "image"))
			end
		end

		if image.build_commands ~= nil then
			local last_stamp = nil
			for _, cmd in ipairs(image.build_commands) do
				local cmd_stamp = stamp_path(image, "build." .. hash_str(cmd))
				dag:add(cmd_stamp, command_build(image, image.build, cmd, cmd_stamp), last_stamp)
				if image.build ~= nil then
					dag:depend(cmd_stamp, stamp_path(image.build, "image"))
				end
				dag:depend(cmd_stamp, stamp.prebuild)
				dag:depend(stamp.build, cmd_stamp)
				last_stamp = cmd_stamp
			end
		end

		if image.commands ~= nil then
			local last_stamp = nil
			for _, cmd in ipairs(image.commands) do
				local cmd_stamp = stamp_path(image, "command." .. hash_str(cmd))
				dag:add(cmd_stamp, command_command(image, cmd, cmd_stamp), last_stamp)
				dag:depend(cmd_stamp, stamp.build)
				dag:depend(stamp.image, cmd_stamp)
				last_stamp = cmd_stamp
			end
		end

		if image.files ~= nil then
			for _, file in pairs(image.files) do
				local filename = file:file_name()
				local output_name = output_path(image, "res/" .. hash_str(file.src) .. "/" .. filename)
				if file:is_local() then
					local path = file:path()
					dag:add(output_name, command_copy(path, output_name), path)
				else
					dag:add(output_name, command_download(file, output_name))
				end
				if not file:is_resource() then
					local stage_name = stage_path(image, file.dest)
					dag:add(stage_name, command_copy(output_name, stage_name), output_name)
					if file.mode ~= nil then
						local stage_stamp = stamp_path(image, "file." .. hash_str(file))
						dag:add(stage_stamp, command_chmod(stage_stamp, stage_name, file), stage_name)
						dag:depend(stamp.stage, stage_stamp)
					end
					dag:depend(stamp.stage, stage_name)
				else
					dag:depend(stamp.prebuild, output_name)
				end
			end
		end
	end

	-- print_r(dag)

	dag:write_ninja(io.stdout)
end

function Package(data)
	local image_name = function(name)
		if name == nil then
			return nil
		end

		if name:sub(1, 1) == "." then
			return data.name .. name
		else
			return name
		end
	end

	array_or_single(data, "images")
	expand_replace(data)

	local images = {}
	for k, image in pairs(data.images) do
		image.name = image_name(image.name)
		if image.build ~= nil then
			image.build.name = image_name(image.build.name)
		end
		if image.depends ~= nil then
			for k, dep in pairs(image.depends) do
				dep.name = image_name(dep.name)
			end
		end

		images[image.name] = image
	end
	data.images = images

	data._type = "encage.package"
	return data
end

function Image(data)
	array_or_single(data, "depends")
	array_or_single(data, "files")
	array_or_single(data, "commands")
	array_or_single(data, "build_commands")
	array_or_single(data, "mounts")

	assert_type_array(data.files, "encage.file")
	assert_type_array(data.commands, "encage.command")
	assert_type_array(data.build_commands, "encage.command")
	assert_type_array(data.depends, "encage.depend")
	assert_type_array(data.mounts, "encage.mount")
	assert_type(data.build, "encage.depend")

	data._type = "encage.image"
	return data
end

function File(data)
	if data.mode ~= nil then
		data.mode = tonumber(data.mode, 8)
	end
	data._type = "encage.file"
	function data:is_local()
		return self.src:match("://") == nil
	end
	function data:file_name()
		return self.src:match("^.+/(.+)$")
	end
	function data:is_resource()
		return self.dest == nil or self.dest:sub(1, 1) ~= "/"
	end
	return data
end

function Command(data)
	data._type = "encage.command"
	local is_alphanum
	is_alphanum = function(s)
		if type(s) == "table" then
			for _, v in ipairs(s) do
				if not is_alphanum(v) then
					return false
				end
			end

			return true
		else
			return s:match("^[-_:/.%a%d]*$") ~= nil
		end
	end

	function data:display_string()
		local s = ""
		for _, v in ipairs(self) do
			v = v:gsub("\9", " "):gsub("\n", "; "):gsub(" +", " ")
			if #s > 0 then
				s = s .. " "
			end

			s = s .. v
		end

		return s
	end

	function data:shell_string()
		local s = ""
		for _, v in ipairs(self) do
			if #s > 0 then
				s = s .. " "
			end
			if is_alphanum(v) then
				s = s .. v
			else
				v = v:gsub("\9", "    ")
				s = s .. string.format("%q", v)
			end
		end

		return s
	end

	function data:shellish_string()
		if is_alphanum(self) then
			return self:shell_string()
		end

		local enc = function(data)
			local out = ""
			for i = 1, #data do
				local c = data:sub(i, i)
				if not is_alphanum(c) then
					c = string.format("\\x%02x", string.byte(data, i))
				end
				out = out .. c
			end

			return out
		end

		local command
		local args = ""
		for i, v in ipairs(self) do
			if i == 1 then
				command = v
			else
				if #args > 0 then
					args = args .. "\0"
				end
				args = args .. v
			end
		end

		return "printf -- '" .. enc(args) .. "' | xargs -0x " .. command
	end

	return data
end

function Depend(data)
	if type(data) == "string" then
		data = { name = data }
	end

	data._type = "encage.depend"
	return data
end

function ShellCommand(command)
	local desc = nil
	if type(command) == "table" then
		desc = command.desc
		command = command[1]
	end

	return Command {
		desc = desc,
		"sh",
		"-ec",
		command,
	}
end

function Mount(data)
	data._type = "encage.mount"
	return data
end

function system_arch()
	local f = io.popen("uname -m")
	local arch = f:read("*l")
	f:close()
	return arch or "x86_64"
end

--
-- Example Config
--

local arch = system_arch()

local config = {
	busybox = {
		arch = arch,
	},
	arch = {
		version = "2016.02.01",
		arch = arch,
		mirrorlist = "https://www.archlinux.org/mirrorlist/?country=all&protocol=https&ip_version=4&ip_version=6&use_mirror_status=on",
		-- base = "base",
		-- base_devel = "base-devel",
		base = "bash bzip2 coreutils diffutils file filesystem findutils gawk gcc-libs glibc grep gzip inetutils iproute2 iputils less logrotate netctl perl procps-ng psmisc s-nail sed shadow tar util-linux which",
		base_devel = "autoconf automake binutils bison fakeroot flex gcc gettext groff gzip libtool m4 make patch pkg-config sed texinfo",
	},
	cdebootstrap = {
		version = "0.7.1",
		arch = "amd64",
	}
}

local package = Package {
	name = "mx.arcn.hello",
	version = "0.0.1",
	images = {
		Image {
			name = ".busybox",
			vars = {
				arch = config.busybox.arch,
			},
			files = {
				File {
					dest = "/sbin/busybox",
					src = "http://www.busybox.net/downloads/binaries/busybox-%{vars.arch}",
					mode = "6755",
					uid = "0",
				},
			},
			commands = Command {
				desc = "populate busybox",
				"/sbin/busybox",
				"sh",
				"-ec",
				[[
				/sbin/busybox mkdir -p "%{path.root}/bin" "%{path.root}/usr/bin" "%{path.root}/usr/sbin"
				/sbin/busybox --install
				]]
			},
		},
		Image {
			name = ".alpine.bootstrap",
			files = {
				File {
					src = "http://dl-4.alpinelinux.org/alpine/latest-stable/main/x86_64/apk-tools-static-2.6.5-r1.apk",
					dest = "apk-tools-static.apk",
				}
			},
			build = Depend ".busybox",
			build_commands = ShellCommand {
				desc = "untar apk-tools",
				[[
				tar -xzf -C "%{path.target}" "%{path.file apk-tools-static.apk}"
				]]
			},
		},
		Image {
			name = ".cdebootstrap",
			build = Depend ".busybox",
			vars = {
				version = config.cdebootstrap.version,
				arch = config.cdebootstrap.arch,
			},
			files = {
				File {
					src = "http://ftp.debian.org/debian/pool/main/c/cdebootstrap/cdebootstrap-static_%{vars.version}_%{vars.arch}.deb",
					dest = "cdebootstrap-static.deb",
				},
			},
			build_commands = ShellCommand {
				desc = "untar bootstrap",
				[[
				ar -p "%{path.file cdebootstrap-static.deb}" data.tar.xz | tar -xj -C "%{path.target}"
				ln -s cdebootstrap-static "%{path.target}/usr/bin/cdebootstrap
				]]
			},
			--[[mounts = Mount {
				kind = "bind",
				source = "%{path.build}/cdebootstrap-helper",
				target = "%{path.root}/wat"
			},]]
		},
		Image {
			name = ".arch-bootstrap",
			vars = {
				version = config.arch.version,
				arch = config.arch.arch,
				mirrorlist = config.arch.mirrorlist,
			},
			files = {
				File {
					src = "http://mirrors.kernel.org/archlinux/iso/%{vars.version}/archlinux-bootstrap-%{vars.version}-%{vars.arch}.tar.gz",
					download_cont = true,
					dest = "arch-bootstrap.tgz",
				},
				File {
					src = "%{vars.mirrorlist}",
					dest = "mirrorlist",
				},
			},
			build = Depend ".busybox",
			build_commands = ShellCommand {
				desc = "untar bootstrap",
				[[
				rm -rf "%{path.target}"/*
				tar -xf "%{path.file arch-bootstrap.tgz}" -C "%{path.target}"
				mv "%{path.target}/root.%{vars.arch}/"* "%{path.target}"
				rmdir "%{path.target}/root.%{vars.arch}"
				]],
			},
			commands = {
				ShellCommand {
					desc = "rankmirrors",
					[[
					perl -pe 's/^#Server/Server/' < "%{path.file mirrorlist}" > /tmp/mirrorlist
					rankmirrors /tmp/mirrorlist > /etc/pacman.d/mirrorlist
					]]
				},
				ShellCommand {
					desc = "pacman-key",
					[[
					pacman-key --init
					pacman-key --populate archlinux
					]],
				},
			},
			mounts = Mount {
				kind = "bind",
				source = "%{path.build}/pacman-cache",
				target = "%{path.root}/var/cache/pacman/pkg"
			},
		},
		Image {
			name = ".arch",
			vars = {
				base = config.arch.base,
			},
			build = Depend ".arch-bootstrap",
			build_commands = ShellCommand {
				desc = "pacstrap",
				[[
				pacstrap -dc "%{path.target}" %{vars.base}
				]],
			},
		},
		Image {
			name = ".arch-devel",
			depends = Depend ".arch",
			vars = {
				base_devel = config.arch.base_devel,
			},
			build = Depend ".arch-bootstrap",
			build_commands = ShellCommand {
				desc = "pacstrap",
				[[
				pacstrap -dc "%{path.target}" %{vars.base_devel}
				]],
			},
		},
		--[=[Image {
			name = ".debian.jessie",
			build = Depend ".cdebootstrap",
			build_commands = ShellCommand {
				desc = "cdebootstrap",
				[[
				cdebootstrap -f minimal jessie "%{path.target}"
				]],
			},
		},]=]
	},
}

return Encage(package)
