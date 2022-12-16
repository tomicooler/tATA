#ifndef TATA_LIB_EXECUTOR
#define TATA_LIB_EXECUTOR

#include <string>

namespace AT {
constexpr char OK[] = "OK\r\n";
}

template <typename T>
concept CExecutor = requires(T o) {
  { o.execute(std::string{}) } -> std::same_as<std::string>;
};

struct NoopExecutor {
  std::string execute(const std::string &c) { return c; };
};

static_assert(CExecutor<NoopExecutor>);

[[nodiscard]] bool isSuccessfulReturn(const std::string &str) {
  return str.ends_with(AT::OK);
}

#endif // TATA_LIB_EXECUTOR
