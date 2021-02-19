/*
 * SGX enclave linker script:
 * As per the SGX Linux SDK Reference, "a linker script is also recommended to hide all unnecessary symbols".
 */

enclave.so
{
    global:
        enclave_entry;
        g_global_data_sim;
        g_global_data;
    local:
        *;
};
