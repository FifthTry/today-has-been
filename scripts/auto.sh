# shellcheck disable=SC2155
export PROJ_ROOT=$(pwd)

export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
#export DATABASE_URL=${DATABASE_URL:-postgresql://127.0.0.1/fifthtry}
export SITE_URL=${SITE_URL:-http://127.0.0.1:8000}

function pushd2() {
    PUSHED=$(pwd)
    cd "${PROJDIR}""$1" >> /dev/null || return
}

function popd2() {
    cd "${PUSHED:-$PROJDIR}" >> /dev/null || return
    unset PUSHED
}


function build-wasm() {
    pushd2 "${PROJ_ROOT}/backend" || return 1
    # cargo clean
    WASMTIME_BACKTRACE_DETAILS=1 cargo build --target wasm32-unknown-unknown --release || return 1
    if ! command -v wasm-pack &> /dev/null; then
        cargo install wasm-opt --locked
    fi
    wasm-opt -O3 -o ../frontend/backend.wasm ../target/wasm32-unknown-unknown/release/backend.wasm
#    cp ../target/wasm32-unknown-unknown/release/backend.wasm ../frontend/ || return 1
    popd2
}


function create-schema() {
    pushd2 "${PROJ_ROOT}"
    if ! command -v diesel &> /dev/null; then
          cargo install diesel_cli --no-default-features --features postgres
    fi

    diesel print-schema --database-url="${DATABASE_URL}" > /tmp/schema.rs
    # if content of ../ft-common/src/schema.rs is different from /tmp/schema.rs, then only copy
    if ! diff -q backend/src/schema.rs /tmp/schema.rs; then
      cp /tmp/schema.rs backend/src/schema.rs
    fi

    popd2
}


function upload-frontend() {
    pushd2 "${PROJ_ROOT}/frontend" || return 1

    if [ -z "$(git status --porcelain)" ]; then
      echo "working directory clean"
    else
      echo "working directory dirty"
      return 1
    fi

    FIFTHTRY_SITE_WRITE_TOKEN=$(cat ../token.txt) \
      DEBUG_API_FIFTHTRY_COM=https://www.fifthtry.com \
      fastn upload todayhasbeen-dotcom

    popd2
}
