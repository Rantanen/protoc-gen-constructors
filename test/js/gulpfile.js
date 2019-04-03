
const { exec } = require('child_process');
const gulp = require('gulp');
const gulp_clean = require('gulp-clean');

function clean() {
    return gulp.src('build/*.js', {read:false}).pipe(gulp_clean());
}

function build(cb) {

    exec( './node_modules/protoc/protoc/bin/protoc ' +
        '--plugin=../../target/debug/protoc-gen-js_constructors ' +
        '--js_out=import_style=commonjs:build ' +
        '--js_constructors_out=../protos/test1.spec,../protos/test2.spec:build ' +
        '-I../protos ' +
        '../protos/test1.proto ' +
        '../protos/test2.proto', cb );
}

exports.clean = clean;
exports.default = exports.build = build;
