#include "Types.hpp"
#include <unordered_map>
#include <string>

class AurenWindowManager {
    private:
    std::unordered_map<AurenID, AurenWindow> _windows;
    VkInstance _instance;

    AurenID findSmallestPossibleID();
    public:

    AurenWindowManager(VkInstance instance);
    ~AurenWindowManager();

    void createWindow(const std::string& title, const AurenVec2D& size, AurenID id = 0);

    AurenWindow& refWindowByID(AurenID id) {
        return _windows[id];
    }

    bool WindowExistsByID(AurenID id) {
        return _windows.contains(id);
    }
};

AurenWindowManager::AurenWindowManager(VkInstance instance) {
    _instance = instance;
}

void AurenWindowManager::createWindow(const std::string& title, const AurenVec2D& size, AurenID id) {
    

    _windows.try_emplace(id, title, size, _instance);
}

AurenID AurenWindowManager::findSmallestPossibleID() {
    AurenID i = 0;
    for (; _windows.contains(i); ++i);
    return i;
}