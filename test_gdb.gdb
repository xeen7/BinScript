set pagination off
break ts_rt_stubs::slab::fast_free_shared
commands
  silent
  printf "fast_free_shared called:\n"
  backtrace 5
  continue
end
break ts_rt_stubs::circ::circ_dec
commands
  silent
  printf "circ_dec called:\n"
  backtrace 5
  continue
end
run
