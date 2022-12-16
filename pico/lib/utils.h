#ifndef TATA_LIB_UTILS
#define TATA_LIB_UTILS

#include <ranges>
#include <string>

namespace Utils {

template <typename T>
void tokenize(T &container, const std::string &str,
              std::string_view delimiter) {
  for (const auto word : std::views::split(str, delimiter)) {
    container.push_back(std::string{word.begin(), word.end()});
  }
}

} // namespace Utils

#endif // TATA_LIB_UTILS
