function(ENCAGE_TARGET_DIR output target)
	set(${output} "build/${target}")
endfunction()

function(ENCAGE_DATA_DIR output target)
	set(${output} "build/${target}/data")
endfunction()

function(ENCAGE_STAMP_DIR output target)
	set(${output} "build/${target}/stamp")
endfunction()

function(ENCAGE_STAGE_DIR output target)
	set(${output} "build/${target}/stamp")
endfunction()

function(ENCAGE_STAMP output target filename)
	ENCAGE_STAMP_DIR(${output} ${target})
	set(${output} ${${output}}/${filename})
endfunction()

function(ENCAGE_RESOURCE output target filename)
	ENCAGE_RESOURCE_DIR(${output} ${target})
	set(${output} ${${output}}/${filename})
endfunction()

function(ENCAGE_DEPENDS(target stamp)

function(ENCAGE_TARGET target)
	ENCAGE_STAMP(output ${target} image)
	add_custom_command(OUTPUT ${output}
		COMMAND touch ${output}
	)
endfunction()

function(ENCAGE_DOWNLOAD target url path)
	execute_process(
		COMMAND basename ${url}
		OUTPUT_VARIABLE filename
	)
	ENCAGE_RESOURCE(resource ${target} ${filename})
	add_custom_command(OUTPUT ${resource}
		COMMAND curl -L -o ${resource} ${url}
	)
	ENCAGE_DEPENDS(${target} image ${resource})
	ENCAGE_DEPENDS(${target} stage ${resource})
endfunction()

function(ENCAGE_COMMAND target command)

endfunction()

function(ENCAGE_HASH output)
	execute_process(
		COMMAND sh -c "echo \"${ARGN}\" | md5sum | cut -d' ' -f1"
		OUTPUT_VARIABLE ${output}
	)
endfunction()
