#include <cstdint>

extern "C" {
const char* u_errorName(int32_t code);
int32_t u_shapeArabic(
    const char16_t* source,
    int32_t source_length,
    char16_t* dest,
    int32_t dest_size,
    uint32_t options,
    int32_t* error_code
);
void ubidi_close(void* bidi);
int32_t ubidi_countParagraphs(const void* bidi, int32_t* error_code);
int32_t ubidi_countRuns(const void* bidi, int32_t* error_code);
int32_t ubidi_getParagraphByIndex(
    const void* bidi,
    int32_t para_index,
    int32_t* para_start,
    int32_t* para_limit,
    uint8_t* para_level,
    int32_t* error_code
);
int32_t ubidi_getProcessedLength(const void* bidi);
int32_t ubidi_getVisualRun(
    void* bidi,
    int32_t run_index,
    int32_t* logical_start,
    int32_t* length
);
void* ubidi_open(void);
void* ubidi_setLine(
    const void* para_bidi,
    int32_t start,
    int32_t limit,
    void* line_bidi,
    int32_t* error_code
);
void ubidi_setPara(
    void* bidi,
    const char16_t* text,
    int32_t length,
    uint8_t para_level,
    const uint8_t* embedding_levels,
    int32_t* error_code
);
int32_t ubidi_writeReordered(
    void* bidi,
    char16_t* dest,
    int32_t dest_size,
    uint16_t options,
    int32_t* error_code
);
int32_t ubidi_writeReverse(
    const char16_t* source,
    int32_t source_length,
    char16_t* dest,
    int32_t dest_size,
    uint16_t options,
    int32_t* error_code
);
}

extern "C" const char* u_errorName_61(int32_t code) {
    return u_errorName(code);
}

extern "C" int32_t u_shapeArabic_61(
    const char16_t* source,
    int32_t source_length,
    char16_t* dest,
    int32_t dest_size,
    uint32_t options,
    int32_t* error_code
) {
    return u_shapeArabic(source, source_length, dest, dest_size, options, error_code);
}

extern "C" void ubidi_close_61(void* bidi) {
    ubidi_close(bidi);
}

extern "C" int32_t ubidi_countParagraphs_61(const void* bidi, int32_t* error_code) {
    return ubidi_countParagraphs(bidi, error_code);
}

extern "C" int32_t ubidi_countRuns_61(const void* bidi, int32_t* error_code) {
    return ubidi_countRuns(bidi, error_code);
}

extern "C" int32_t ubidi_getParagraphByIndex_61(
    const void* bidi,
    int32_t para_index,
    int32_t* para_start,
    int32_t* para_limit,
    uint8_t* para_level,
    int32_t* error_code
) {
    return ubidi_getParagraphByIndex(
        bidi,
        para_index,
        para_start,
        para_limit,
        para_level,
        error_code
    );
}

extern "C" int32_t ubidi_getProcessedLength_61(const void* bidi) {
    return ubidi_getProcessedLength(bidi);
}

extern "C" int32_t ubidi_getVisualRun_61(
    void* bidi,
    int32_t run_index,
    int32_t* logical_start,
    int32_t* length
) {
    return ubidi_getVisualRun(bidi, run_index, logical_start, length);
}

extern "C" void* ubidi_open_61(void) {
    return ubidi_open();
}

extern "C" void* ubidi_setLine_61(
    const void* para_bidi,
    int32_t start,
    int32_t limit,
    void* line_bidi,
    int32_t* error_code
) {
    return ubidi_setLine(para_bidi, start, limit, line_bidi, error_code);
}

extern "C" void ubidi_setPara_61(
    void* bidi,
    const char16_t* text,
    int32_t length,
    uint8_t para_level,
    const uint8_t* embedding_levels,
    int32_t* error_code
) {
    ubidi_setPara(bidi, text, length, para_level, embedding_levels, error_code);
}

extern "C" int32_t ubidi_writeReordered_61(
    void* bidi,
    char16_t* dest,
    int32_t dest_size,
    uint16_t options,
    int32_t* error_code
) {
    return ubidi_writeReordered(bidi, dest, dest_size, options, error_code);
}

extern "C" int32_t ubidi_writeReverse_61(
    const char16_t* source,
    int32_t source_length,
    char16_t* dest,
    int32_t dest_size,
    uint16_t options,
    int32_t* error_code
) {
    return ubidi_writeReverse(
        source,
        source_length,
        dest,
        dest_size,
        options,
        error_code
    );
}
