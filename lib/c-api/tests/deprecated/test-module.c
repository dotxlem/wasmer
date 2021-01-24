#include <stdio.h>
#include "wasmer.h"
#include <assert.h>
#include <stdint.h>

int main()
{
    // Read the wasm file bytes
    FILE *file = fopen("assets/sum.wasm", "r");
    fseek(file, 0, SEEK_END);
    long len = ftell(file);
    uint8_t *bytes = malloc(len);
    fseek(file, 0, SEEK_SET);
    fread(bytes, 1, len, file);
    fclose(file);

    wasmer_module_t *module = NULL;
    wasmer_result_t compile_result = wasmer_compile(&module, bytes, len);
    printf("Compile result:  %d\n", compile_result);
    assert(compile_result == WASMER_OK);

    wasmer_import_t imports[] = {};
    wasmer_instance_t *instance = NULL;
    wasmer_result_t instantiate_result = wasmer_module_instantiate(module, &instance, imports, 0);
    printf("Instantiate result:  %d\n", compile_result);
    assert(instantiate_result == WASMER_OK);

    wasmer_value_t param_one;
    param_one.tag = WASM_I32;
    param_one.value.I32 = 7;
    wasmer_value_t param_two;
    param_two.tag = WASM_I32;
    param_two.value.I32 = 8;
    wasmer_value_t params[] = {param_one, param_two};

    wasmer_value_t result_one;
    wasmer_value_t results[] = {result_one};

    wasmer_result_t call_result = wasmer_instance_call(instance, "sum", params, 2, results, 1);
    printf("Call result:  %d\n", call_result);
    printf("Result: %d\n", results[0].value.I32);
    assert(results[0].value.I32 == 15);
    assert(call_result == WASMER_OK);

    printf("Destroy instance\n");
    wasmer_instance_destroy(instance);

    printf("Destroy module\n");
    wasmer_module_destroy(module);
    return 0;
}
