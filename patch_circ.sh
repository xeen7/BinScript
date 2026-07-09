cat << 'INNER_EOF' > /tmp/sed_script_circ.sed
s/let tag = tagged & TAG_MASK;/let tag = tagged & crate::dynamic_call::helpers::TAG_MASK;/
s/if tag == crate::dynamic_call::dispatchers::TAG_OWNED {/if tag == crate::dynamic_call::helpers::TAG_OWNED {/
s/crate::core::alloc::__bs_drop_owned((tagged & PAYLOAD_MASK) as \*mut u8);/crate::core::alloc::__bs_drop_owned((tagged & crate::dynamic_call::helpers::PAYLOAD_MASK) as \*mut u8);/
INNER_EOF
sed -i -f /tmp/sed_script_circ.sed rt-stubs/src/circ.rs
